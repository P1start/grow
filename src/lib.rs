#![feature(alloc, heap_api, coerce_unsized, unsize, specialization, unique, oom)]
extern crate alloc;
use std::mem;
use std::ptr::{self, Unique};
use std::cmp;
use alloc::heap;
use std::ops::{CoerceUnsized, Deref, DerefMut};
use std::marker::Unsize;

pub struct Grow<T: ?Sized> {
    _ptr: Unique<T>,
    _capacity: usize,
}

fn decompose_ptr<T: ?Sized>(mut ptr: &mut *mut T) -> (*mut u8, Option<&mut usize>) {
    unsafe {
        if mem::size_of::<*mut T>() == mem::size_of::<usize>() {
            (*mem::transmute::<&mut *mut T, &mut *mut u8>(ptr), None)
        } else {
            let &mut (ptr, ref mut fat) = mem::transmute::<&mut *mut T, &mut (*mut u8, usize)>(ptr);
            (ptr, Some(fat))
        }
    }
}

fn make_ptr<T: ?Sized>(ptr: *mut u8, fatness: Option<usize>) -> *mut T {
    let fatsize = mem::size_of::<usize>() * (fatness.is_some() as usize + 1);
    debug_assert_eq!(mem::size_of::<*mut T>(), fatsize);
    unsafe {
        if let Some(fat) = fatness {
            *mem::transmute::<&(*mut u8, usize), &*mut T>(&(ptr, fat))
        } else {
            *mem::transmute::<&*mut u8, &*mut T>(&ptr)
        }
    }
}

#[inline(never)]
unsafe fn alloc_or_realloc<T: ?Sized>(mut ptr: *mut T, size: usize, old_size: usize) -> *mut T {
    let (ptr, fat) = decompose_ptr(&mut ptr);
    let ptr = if old_size == 0 {
        heap::allocate(size, mem::align_of_val(&*ptr))
    } else {
        heap::reallocate(ptr, old_size, size, mem::align_of_val(&*ptr))
    };
    make_ptr(ptr, fat.map(|x| *x))
}

impl<T> Grow<T> {
    pub fn new(v: T) -> Grow<T> {
        unsafe {
            let size = mem::size_of::<T>();
            let ptr = if size == 0 {
                heap::EMPTY as *mut T
            } else {
                heap::allocate(size, mem::align_of::<T>()) as *mut T
            };
            if ptr.is_null() { panic!("allocation failure"); }
            ptr::write(ptr, v);
            Grow {
                _ptr: Unique::new(ptr),
                _capacity: size,
            }
        }
    }

    pub fn with_capacity(v: T, capacity: usize) -> Grow<T> {
        unsafe {
            let size = cmp::max(capacity, mem::size_of::<T>());
            let ptr = if size == 0 {
                heap::EMPTY as *mut T
            } else {
                heap::allocate(size, mem::align_of::<T>()) as *mut T
            };
            if ptr.is_null() { panic!("allocation failure"); }
            ptr::write(ptr, v);
            Grow {
                _ptr: Unique::new(ptr),
                _capacity: size,
            }
        }
    }
}

impl<T: ?Sized> Grow<T> {
    // FIXME: doesn't seem to work with Grow::grow()
    /*pub fn from_box(mut v: Box<T>) -> Grow<T> {
        let size = mem::size_of_val(&*v);
        Grow {
            _ptr: unsafe { Unique::new(&mut *v) },
            _capacity: size,
        }
    }*/

    /// Allocate space for `size` bytes of data.
    pub fn grow(&mut self, size: usize) {
        unsafe {
            if size > self._capacity {
                let mut ptr = alloc_or_realloc(self._ptr.get_mut(), size, self._capacity);
                if decompose_ptr(&mut ptr).0.is_null() {
                    alloc::oom::oom()
                }
                self._ptr = Unique::new(ptr);
                self._capacity = size;
            }
        }
    }

    pub fn capacity_bytes(&self) -> usize {
        self._capacity
    }

    /// Sets the inner contents of the box to `val`, reallocating if necessary.
    pub fn set<U: Unsize<T>>(&mut self, mut val: U) {
        let size = mem::size_of_val(&val);
        self.grow(size);
        let mut vptr: *mut T = &mut val;
        let selfptr: *mut T = *self._ptr;
        let (vptr, vsize) = decompose_ptr(&mut vptr);
        unsafe {
            let s = mem::transmute::<&mut Unique<T>, &mut *mut T>(&mut self._ptr);
            let (sptr, ssize) = decompose_ptr(s);
            ptr::drop_in_place(selfptr);
            ptr::copy(vptr, sptr, size);
            // This should never really fail
            if let (Some(vsize), Some(ssize)) = (vsize, ssize) {
                ptr::copy(vsize, ssize, 1);
            }
        }
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Grow<U>> for Grow<T> {}

impl<T: ?Sized> Deref for Grow<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            self._ptr.get()
        }
    }
}

impl<T: ?Sized> DerefMut for Grow<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            self._ptr.get_mut()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set() {
        let mut g: Grow<[i32]> = Grow::new([7, 8, 9]);
        g.set([1, 2, 3, 4, 5, 6]);

        let x: &[i32] = &g;
        assert_eq!(x.len(), 6);
        for (i, v) in x.iter().enumerate() {
            assert_eq!(*v, match i {
                0 => 1,
                1 => 2,
                2 => 3,
                3 => 4,
                4 => 5,
                5 => 6,
                _ => unreachable!(),
            });
        }


        let mut a = 0;
        {
            let mut g: Grow<FnMut(i32) -> i32> = Grow::new(|x| x + 1);
            assert_eq!((*&mut *g)(1), 2);
            assert_eq!(g.capacity_bytes(), 0);
            g.set(|x| {
                a += 1;
                x - 1
            });
            assert_eq!((*&mut *g)(1), 0);
            assert_eq!(g.capacity_bytes(), ::std::mem::size_of::<&mut i32>());
        }
        assert_eq!(a, 1);
    }

    #[test]
    fn grow() {
        let mut g: Grow<[i32; 3]> = Grow::new([1, 2, 3]);
        assert_eq!(g.capacity_bytes(), 12);
        g.grow(64);
        assert_eq!(g.capacity_bytes(), 64);
    }

    #[test]
    fn with_capacity() {
        let g: Grow<[i32; 3]> = Grow::with_capacity([1, 2, 3], 64);
        assert_eq!(g.capacity_bytes(), 64);
    }
}
