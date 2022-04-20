#![allow(non_snake_case)]
use std::marker::PhantomData;


#[derive(Clone, Debug, Default)]
pub struct MBC1<'a> {
            phantom: PhantomData<&'a u8>,

}

impl<'a> MBC1<'a>{

    pub fn new() -> MBC1<'a> {
       let mbc1 = MBC1{
            phantom: PhantomData,
       };
       mbc1
    }
}

