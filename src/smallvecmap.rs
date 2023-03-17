//! 实现数据结构`SmallVecMap`, 并为`SmallVecMap`实现了`Map<K=u32,V=T>`
//! 就像其名字描述的一样，`SmallVecMap`以Vec作为数据结构，实现索引到值得映射。
//! 
//! SmallVecMap通常用于存放的少量数据，数据的key可以跨度比较大。
//! 再决定使用SmallVecMap前，你应该综合考虑这几个问题：访问性能、数据连续性、内存的浪费情况。
//!
use std::mem::replace;
use std::fmt::{Debug};
use std::ops::{Index, IndexMut};
use std::slice::{Iter, IterMut};
use smallvec::{SmallVec, Array};
use pi_null::Null;


use crate::Map;

pub struct Arr<T, const N: usize>([(T, u32); N]);

unsafe impl<T, const N: usize> Array for Arr<T, N> {
    type Item = (T, u32);

    fn size() -> usize {
        N
    }
}

/// 数据结构SmallVecMap
#[derive(Debug, Hash, Clone)]
pub struct SmallVecMap<T, const N: usize> {
    indexs: Vec<u32>,// Chunk of memory
    entries: SmallVec<Arr<T, N>>,// Chunk of memory
}

impl<T, const N: usize> Default for SmallVecMap<T, N> {
    fn default() -> Self {
        SmallVecMap::new()
    }
}
// impl<T: Clone, const N: usize> Clone for SmallVecMap<T, N> {
//     fn clone(&self) -> Self {
//         SmallVecMap {
//             indexs: self.indexs.clone(),
//             entries: self.entries.clone(),
//         }
//     }
// }
impl<T, const N: usize> SmallVecMap<T, N> {

    /// 创建一个SmallVecMap实例
    pub fn new() -> Self {
        SmallVecMap::with_capacity(0)
    }
 
    /// 创建一个SmallVecMap实例, 并指定初始化容量
    pub fn with_capacity(capacity: usize) -> SmallVecMap<T, N> {
        SmallVecMap {
            indexs: Vec::with_capacity(capacity),
            entries: SmallVec::new(),
        }
    }

    /// 获取SmallVecMap当前的容量
    pub fn capacity(&self) -> usize {
        self.indexs.capacity()
    }

    /// 扩充容量
    pub fn reserve(&mut self, additional: usize) {
        self.indexs.reserve(additional);
    }

    /// 扩充容量
    pub fn reserve_exact(&mut self, additional: usize) {
        self.indexs.reserve_exact(additional);
    }
    
    /// 清空数据
    pub fn clear(&mut self) {
        self.indexs.clear();
        self.entries.clear();
    }

    /// 片段当前是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// 获取一个只读迭代器，可以获取值所对应的index
    pub fn iter(&self) -> Iter<'_, (T, u32)> {
        self.entries.iter()
    }
    /// 获取一个可写迭代器，可以获取值所对应的index
    pub fn iter_mut(&mut self) -> IterMut<'_, (T, u32)> {
        self.entries.iter_mut()
    }
    
    /// 替换指定位置的值, 并返回旧值。你应该确认，旧值一定存在，否则将会panic
    pub unsafe fn replace(&mut self, index: u32, val: T) -> T {
        replace(self.get_unchecked_mut(index), val)
    }
    /// 取到某个偏移位置的只读值
    pub fn get(&self, index: u32) -> Option<&T> {
        if index as usize >= self.entries.len() {
            return None;
        }
        let i = self.indexs[index as usize];
        if i.is_null() {
            return None;
        }
        Some(&self.entries[i as usize].0)
    }

    /// 取到某个偏移位置的可变值
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        if index as usize >= self.entries.len(){
            return None;
        }
        let i = self.indexs[index as usize];
        if i.is_null() {
            return None;
        }
        Some(&mut self.entries[i as usize].0)
    }

    /// 取到某个偏移位置的只读值
    /// 如果该位置不存在值，将panic
    pub unsafe fn get_unchecked(&self, index: u32) -> &T {
        &self.entries[self.indexs[index as usize] as usize].0
    }

    /// 取到某个偏移位置的可变值
    /// 如果该位置不存在值，将panic
    pub unsafe fn get_unchecked_mut(&mut self, index: u32) -> &mut T {
        &mut self.entries[self.indexs[index as usize] as usize].0
    }

    /// 在指定位置插入一个值，并返回旧值，如果不存在旧值，返回None
    pub fn insert(&mut self, index:u32, val: T) -> Option<T>{
		let len = self.indexs.len();
        if index as usize > self.indexs.capacity() {
            self.indexs.reserve(index as usize - self.indexs.capacity());
            self.indexs.extend((0..index as usize - len + 1).map(|_| u32::null()));
            self.indexs[index as usize] = self.entries.len() as u32;
			self.entries.push((val, index));
            None
		} else if index as usize > len {
            self.indexs.extend((0..index as usize - len + 1).map(|_| u32::null()));
            self.indexs[index as usize] = self.entries.len() as u32;
			self.entries.push((val, index));
            None
		} else if index as usize == len {
			self.indexs.push(self.entries.len() as u32);
			self.entries.push((val, index));
            None
        } else {
            let i = unsafe {self.indexs.get_unchecked_mut(index as usize)};
            if (*i).is_null() {
                *i = index;
                self.entries.push((val, index));
                return None;
            }
            Some(replace(&mut self.entries[*i as usize].0, val))
        }
    }

    /// 移除指定位置的值，返回被移除的值，如果该位置不存在一个值，返回None
    pub fn remove(&mut self, index: u32) -> Option<T> {
        if index as usize >= self.indexs.len() {
            return None;
        }
        let i = unsafe {self.indexs.get_unchecked_mut(index as usize)};
        if (*i).is_null() {
            return None
        }
        if *i as usize + 1 == self.entries.len() {
            *i = u32::null();
            return Some(self.entries.pop().unwrap().0)
        }
        let i = replace(i, u32::null()) as usize;
        // 从尾部交换元素到指定位置
        let r = Some(self.entries.swap_remove(i).0);
        // 修复索引
        self.indexs[self.entries[i].1 as usize] = i as u32;
        r
    }

    /// 移除指定位置的值，返回被移除的值，如果该位置不存在一个值将panic
    pub unsafe fn remove_unchecked(&mut self, index: u32) -> T {
        let i = &mut self.indexs[index as usize];
        if *i as usize + 1 == self.entries.len() {
            *i = u32::null();
            return self.entries.pop().unwrap().0
        }
        let i = replace(i, u32::null()) as usize;
        // 从尾部交换元素到指定位置
        let r = self.entries.swap_remove(i).0;
        // 修复索引
        self.indexs[self.entries[i].1 as usize] = i as u32;
        r
    }

    /// 判断指定位置是否存在一个值
    pub fn contains(&self, index: u32) -> bool {
        if index as usize >= self.indexs.len(){
            return false;
        }
        return !self.indexs[index as usize].is_null()
    }

    /// 取到SmallVecMap的长度
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}


/// 为SmallVecMap实现Map
impl<T, const N: usize> Map for SmallVecMap<T, N> {
	type Key = u32;
	type Val = T;
    #[inline]
    fn get(&self, key: &Self::Key) -> Option<&T> {
        self.get(*key)
    }

    #[inline]
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut T> {
        self.get_mut(*key)
    }

    #[inline]
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &T {
        self.get_unchecked(*key)
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut T {
        self.get_unchecked_mut(*key)
    }

    #[inline]
    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> T {
        self.remove_unchecked(*key)
    }

    #[inline]
    fn insert(&mut self, key: Self::Key, val: T) -> Option<T> {
        self.insert(key, val)
    }

    #[inline]
    fn remove(&mut self, key: &Self::Key) -> Option<T> {
        self.remove(*key)
    }

    #[inline]
    fn contains(&self, key: &Self::Key) -> bool {
        self.contains(*key)
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline]
    fn capacity(&self) -> usize {
        self.indexs.capacity()
    }
    #[inline]
    fn mem_size(&self) -> usize {
        self.capacity() * std::mem::size_of::<T>()
	}
	
	fn with_capacity(capacity: usize) -> Self {
		SmallVecMap::with_capacity(capacity)
	}
}

impl<T, const N: usize> Index<usize> for SmallVecMap<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.get(index as u32).unwrap()
    }
}

impl<T, const N: usize> IndexMut<usize> for SmallVecMap<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index as u32).unwrap()
    }
}


#[cfg(test)]
use std::time::Instant;
#[test]
fn test_time(){
    let mut map: SmallVecMap<u32, 16> = SmallVecMap::new();
    let _cc = map.clone();
    let mut arr = Vec::with_capacity(100000);
    let time = Instant::now();
    for i in 0..10000 {
        arr.push(i as f32 + 0.5);
    }
    println!("insert vec time: {:?}", Instant::now() - time);

    let time = Instant::now();
    for i in 1..10001 {
        map.insert(i, i);
    }
    println!("insert SmallVecMap time: {:?}", Instant::now() - time);


    let mut map: SmallVecMap<f32, 16> = SmallVecMap::new();

    let time = Instant::now();
    for i in 1..10001 {
        map.insert(i, 1.0);
    }
    println!("insert SmallVecMap time: {:?}", Instant::now() - time);

}
#[test]
fn test(){
    let mut map: SmallVecMap<u32, 8> = SmallVecMap::new();
    for i in 1..71{
        map.insert(i, i);
        println!("map------{:?}", map);
    }

    map.remove(30);
    println!("r 30------{:?}", map);

    map.remove(31);
    println!("r 31------{:?}", map);

    map.remove(69);
    println!("r 69------{:?}", map);

    map.remove(70);
    println!("r 70------{:?}", map);

    assert_eq!(map.contains(0), false);
    assert_eq!(map.contains(1), true);
    assert_eq!(map.contains(71), false);
    assert_eq!(map.contains(72), false);

    assert_eq!(map.get(0), None);
    assert_eq!(map.get(1), Some(&1));
    assert_eq!(map.get(50), Some(&50));
    assert_eq!(map.get(70), None);
    assert_eq!(map.get(72), None);


    assert_eq!(map.get_mut(0), None);
    assert_eq!(map.get_mut(64), Some(&mut 64));
    assert_eq!(map.get_mut(30), None);
    assert_eq!(map.get_mut(20), Some(&mut 20));
    assert_eq!(map.get_mut(75), None);

    assert_eq!(unsafe{map.get_unchecked(2)}, &2);
    assert_eq!(unsafe{map.get_unchecked(9)}, &9);
    assert_eq!(unsafe{map.get_unchecked(55)}, &55);
    assert_eq!(unsafe{map.get_unchecked(60)}, &60);

    assert_eq!(unsafe{map.get_unchecked_mut(44)}, &mut 44);
    assert_eq!(unsafe{map.get_unchecked_mut(33)}, &mut 33);
    assert_eq!(unsafe{map.get_unchecked_mut(7)}, &mut 7);
}

// #[test]
// fn test_eff(){
    
//     let mut map: SmallVecMap<u64> = SmallVecMap::new();
//     let time = now_millis();
//     for i in 1..1000001{
//         map.insert(i as usize, i);
//     }
//     let time1 = now_millis();
//     println!("insert time-----------------------------------------------{}", time1 - time);

//     for i in 1..1000001{
//         unsafe { map.remove(i) };
//     }
//     let time2 = now_millis();
//     println!("remove time-----------------------------------------------{}", time2 - time1);

//     let mut v = Vec::new();

//     let time3 = now_millis();
//     for i in 1..1000001{
//         v.push(i);
//     }

//     let time4 = now_millis();
//     println!("insert vec time-----------------------------------------------{}", time4 - time3);
// }

// #[test]
// fn m(){
//     //let a: usize = (usize::max_value() - 1) << 1;
//     println!("xxxxxxxxxxxxxxxxxxxxxx");
// }