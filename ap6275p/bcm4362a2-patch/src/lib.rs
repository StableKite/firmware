#![no_std]
//! BCM4362A2 HCD/PatchRAM reconstruction — Stage 5.
pub mod volatile;pub const HCI_WRITE_RAM:u16=0xFC4C;pub const HCI_LAUNCH_RAM:u16=0xFC4E;
pub const PATCH_CODE_START:u32=0x0016_0400;pub const PATCH_CODE_END:u32=0x0016_E7A8;pub const PATCH_DATA_START:u32=0x0022_1D9C;pub const PATCH_DATA_END:u32=0x0022_24CC;pub const PATCH_CONFIG_START:u32=0x0024_0000;pub const PATCH_CONFIG_END:u32=0x0024_262F;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct HcdRecord<'a>{pub opcode:u16,pub payload:&'a[u8]}pub struct HcdIter<'a>{data:&'a[u8],off:usize}impl<'a>HcdIter<'a>{pub const fn new(data:&'a[u8])->Self{Self{data,off:0}}}impl<'a>Iterator for HcdIter<'a>{type Item=HcdRecord<'a>;fn next(&mut self)->Option<Self::Item>{if self.off+3>self.data.len(){return None}let op=u16::from_le_bytes([self.data[self.off],self.data[self.off+1]]);let n=self.data[self.off+2]as usize;let s=self.off+3;let e=s.checked_add(n)?;if e>self.data.len(){self.off=self.data.len();return None}self.off=e;Some(HcdRecord{opcode:op,payload:&self.data[s..e]})}}
impl<'a>HcdRecord<'a>{pub fn write_ram(&self)->Option<(u32,&'a[u8])>{if self.opcode!=HCI_WRITE_RAM||self.payload.len()<4{return None}let a=u32::from_le_bytes(self.payload[0..4].try_into().ok()?);Some((a,&self.payload[4..]))}pub fn launch_ram(&self)->Option<u32>{if self.opcode!=HCI_LAUNCH_RAM||self.payload.len()<4{return None}Some(u32::from_le_bytes(self.payload[0..4].try_into().ok()?))}}
#[repr(C)]pub struct BtObjectHeader{_pad00:[u8;0x14],pub state:u8}
#[repr(C)]pub struct BtField5Message{_pad00:[u8;5],pub field5:u8}
pub fn object_state_is_2(object:Option<&BtObjectHeader>)->bool{matches!(object,Some(x) if x.state==2)}pub const fn index_stride20(index:u8)->u32{20*index as u32}pub const fn u32_gt_2(v:u32)->bool{v>2}pub fn set_field5_to_12(v:&mut BtField5Message){v.field5=12}
pub fn find_first_slot_state1(mut state:impl FnMut(u8)->u32)->u8{let mut i=0u8;while i<8{if state(i)==1{return i}i+=1}8}
#[cfg(test)]extern crate std;#[cfg(test)]mod tests{use super::*;use core::mem::offset_of;#[test]fn hcd(){let x=[0x4c,0xfc,5,0,0,0x24,0,0xaa,0x4e,0xfc,4,0xff,0xff,0xff,0xff];let v:std::vec::Vec<_>=HcdIter::new(&x).collect();assert_eq!(v[0].write_ram().unwrap().0,0x240000);assert_eq!(v[1].launch_ram(),Some(0xffff_ffff));}#[test]fn layouts(){assert_eq!(offset_of!(BtObjectHeader,state),0x14);assert_eq!(offset_of!(BtField5Message,field5),5);}#[test]fn slot(){assert_eq!(find_first_slot_state1(|i|if i==3{1}else{0}),3);assert_eq!(find_first_slot_state1(|_|0),8);}}

/// Stage 6: high-confidence libre replacements for three heavily-used ROM primitives.
pub const ROM_MEMCPY_ADDR:u32=0x0000_3DB4;
pub const ROM_MEMSET_ADDR:u32=0x0000_3D24;
pub const ROM_MEMCMP_ADDR:u32=0x000F_8CAC;

/// # Safety
/// `dst` and `src` must each be valid for `len` bytes and must not overlap,
/// matching the observed memcpy-like primitive.
pub unsafe fn libre_memcpy(dst:*mut u8,src:*const u8,len:usize)->*mut u8{
    let out=dst;
    let mut i=0usize;
    while i<len { unsafe { core::ptr::write(dst.add(i),core::ptr::read(src.add(i))); } i+=1; }
    out
}
/// # Safety
/// `dst` must be valid for `len` writable bytes.
pub unsafe fn libre_memset(dst:*mut u8,value:u8,len:usize)->*mut u8{
    let out=dst; let mut i=0usize;
    while i<len { unsafe { core::ptr::write(dst.add(i),value); } i+=1; }
    out
}
/// # Safety
/// `a` and `b` must each be valid for `len` readable bytes.
pub unsafe fn libre_memcmp(a:*const u8,b:*const u8,len:usize)->i32{
    let mut i=0usize;
    while i<len {
        let av=unsafe{core::ptr::read(a.add(i))}; let bv=unsafe{core::ptr::read(b.add(i))};
        if av!=bv { return av as i32-bv as i32; } i+=1;
    }
    0
}
#[cfg(test)]mod stage6_tests{use super::*;#[test]fn libre_mem_primitives(){let src=[1u8,2,3,4,5,6];let mut dst=[0u8;6];unsafe{libre_memcpy(dst.as_mut_ptr(),src.as_ptr(),6);}assert_eq!(dst,src);unsafe{libre_memset(dst.as_mut_ptr(),0x5a,3);}assert_eq!(&dst[..3],&[0x5a;3]);assert_eq!(unsafe{libre_memcmp(src.as_ptr(),src.as_ptr(),6)},0);assert!(unsafe{libre_memcmp(src.as_ptr(),dst.as_ptr(),6)}<0);}}

/// Stage 7 next ROM targets, ordered by observed call count after Stage-6 resolutions.
pub const STAGE7_NEXT_ROM_TARGETS:&[(u32,u32)]=&[
(0x94C0,44),
(0x1D104,20),
(0x7698A,20),
(0x17E2C,19),
(0x9968C,17),
(0x179F2,15),
(0xBDDBC,15),
(0x771F8,15),
(0x18540,15),
(0x996A0,14),
(0x79DB4,12),
(0x17820,12),
(0x8D278,11),
(0x998A8,11),
(0xA1688,11),
(0x19318,10),
(0x9982C,10),
(0x904,10),
(0x86984,9),
(0x19754,9),
(0x20384,9),
(0x85A5C,9),
(0xB0460,9),
(0x8565C,8),
(0x18060,7),
(0x79A68,7),
(0xB06D0,7),
(0x84738,7),
(0x8256C,7),
(0x2CE90,7),
(0x8D34C,7),
(0x6E774,7)];

/// Stage 8: high-confidence ROM stack-protector failure target.
pub const ROM_STACK_GUARD_FAIL_ADDR:u32=0x0000_94C0;

/// Libre terminal replacement for the ROM stack-canary failure sink.
#[cold]
#[inline(never)]
pub fn libre_stack_guard_fail()->! {
    loop { core::hint::spin_loop(); }
}

#[cfg(test)]
mod stage8_tests {
    use super::*;
    #[test]
    fn stack_guard_target_is_rom() {
        assert_eq!(ROM_STACK_GUARD_FAIL_ADDR, 0x94C0);
    }
}

/// Stage 9: behaviorally identified ROM critical-state swap-like entry.
/// Hardware mechanism remains intentionally unnamed until register-level evidence is frozen.
pub const ROM_CRITICAL_STATE_SWAP_LIKE_ADDR:u32=0x0000_0780;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct CriticalStateExchange{pub enter_value:u32,pub restore_token:u32}
pub const fn critical_state_exchange(restore_token:u32)->CriticalStateExchange{CriticalStateExchange{enter_value:1,restore_token}}
#[cfg(test)]mod stage9_tests{use super::*;#[test]fn exchange_shape(){let e=critical_state_exchange(0x1234);assert_eq!(e.enter_value,1);assert_eq!(e.restore_token,0x1234);assert_eq!(ROM_CRITICAL_STATE_SWAP_LIKE_ADDR,0x780);}}

/// Stage 10: RAII-style abstraction for the Stage-9 critical-state exchange.
/// The hardware primitive itself remains a trait until register semantics are proven.
pub trait CriticalStatePrimitive{fn swap(&mut self,value:u32)->u32;}
pub struct CriticalGuard<'a,P:CriticalStatePrimitive>{primitive:&'a mut P,token:u32}
impl<'a,P:CriticalStatePrimitive> CriticalGuard<'a,P>{pub fn enter(primitive:&'a mut P)->Self{let token=primitive.swap(1);Self{primitive,token}}pub const fn token(&self)->u32{self.token}}
impl<P:CriticalStatePrimitive> Drop for CriticalGuard<'_,P>{fn drop(&mut self){let _=self.primitive.swap(self.token);}}
#[cfg(test)]mod stage10_tests{use super::*;struct Mock{calls:[u32;2],n:usize}impl CriticalStatePrimitive for Mock{fn swap(&mut self,v:u32)->u32{self.calls[self.n]=v;self.n+=1;0x55AA}}#[test]fn guard_restores(){let mut m=Mock{calls:[0;2],n:0};{let g=CriticalGuard::enter(&mut m);assert_eq!(g.token(),0x55AA);}assert_eq!(m.calls,[1,0x55AA]);}}

/// Stage 11: transport-independent application of Broadcom HCD PatchRAM records.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum LaunchAddress{Explicit(u32),Sentinel}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct PatchApplyReport{pub writes:u32,pub bytes:u32,pub launches:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum PatchApplyError<E>{Truncated,Malformed,UnsupportedOpcode(u16),Transport(E)}
pub trait PatchRamSink{type Error;fn write_ram(&mut self,address:u32,data:&[u8])->Result<(),Self::Error>;fn launch_ram(&mut self,address:LaunchAddress)->Result<(),Self::Error>;}
pub fn apply_hcd<S:PatchRamSink>(data:&[u8],sink:&mut S)->Result<PatchApplyReport,PatchApplyError<S::Error>>{
    let mut off=0usize;let mut r=PatchApplyReport::default();
    while off<data.len(){
        if data.len()-off<3{return Err(PatchApplyError::Truncated)}
        let opcode=u16::from_le_bytes([data[off],data[off+1]]);let len=data[off+2] as usize;off+=3;
        if data.len()-off<len{return Err(PatchApplyError::Truncated)}
        let p=&data[off..off+len];off+=len;
        match opcode{
            HCI_WRITE_RAM=>{if p.len()<4{return Err(PatchApplyError::Malformed)}let addr=u32::from_le_bytes([p[0],p[1],p[2],p[3]]);sink.write_ram(addr,&p[4..]).map_err(PatchApplyError::Transport)?;r.writes+=1;r.bytes+=u32::try_from(p.len()-4).unwrap_or(u32::MAX);}
            HCI_LAUNCH_RAM=>{if p.len()!=4{return Err(PatchApplyError::Malformed)}let addr=u32::from_le_bytes([p[0],p[1],p[2],p[3]]);sink.launch_ram(if addr==u32::MAX{LaunchAddress::Sentinel}else{LaunchAddress::Explicit(addr)}).map_err(PatchApplyError::Transport)?;r.launches+=1;}
            x=>return Err(PatchApplyError::UnsupportedOpcode(x)),
        }
    }
    Ok(r)
}
#[cfg(test)]mod stage11_tests{use super::*;#[derive(Default)]struct S{w:u32,b:u32,l:Option<LaunchAddress>}impl PatchRamSink for S{type Error=();fn write_ram(&mut self,_:u32,d:&[u8])->Result<(),()>{self.w+=1;self.b+=d.len()as u32;Ok(())}fn launch_ram(&mut self,a:LaunchAddress)->Result<(),()>{self.l=Some(a);Ok(())}}#[test]fn apply(){let h=[0x4C,0xFC,6,0x00,0x04,0x16,0x00,0xAA,0xBB,0x4E,0xFC,4,0xFF,0xFF,0xFF,0xFF];let mut s=S::default();let r=apply_hcd(&h,&mut s).unwrap();assert_eq!(r,PatchApplyReport{writes:1,bytes:2,launches:1});assert_eq!(s.l,Some(LaunchAddress::Sentinel));}}

/// Stage 12: no-alloc structural reconstruction of the HCD PatchRAM image layout.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct PatchRegion{pub start:u32,pub end:u32}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct PatchLayout{pub regions:[Option<PatchRegion>;3],pub region_count:u8,pub writes:u32,pub bytes:u32,pub sentinel_launches:u8,pub explicit_launches:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum PatchLayoutError{Truncated,Malformed,UnsupportedOpcode(u16),TooManyRegions,AddressOverflow}
pub fn analyze_patch_layout(data:&[u8])->Result<PatchLayout,PatchLayoutError>{let mut o=0usize;let mut l=PatchLayout::default();while o<data.len(){if data.len()-o<3{return Err(PatchLayoutError::Truncated)}let op=u16::from_le_bytes([data[o],data[o+1]]);let n=data[o+2]as usize;o+=3;if data.len()-o<n{return Err(PatchLayoutError::Truncated)}let p=&data[o..o+n];o+=n;match op{HCI_WRITE_RAM=>{if p.len()<4{return Err(PatchLayoutError::Malformed)}let a=u32::from_le_bytes([p[0],p[1],p[2],p[3]]);let e=a.checked_add(u32::try_from(p.len()-4).map_err(|_|PatchLayoutError::AddressOverflow)?).ok_or(PatchLayoutError::AddressOverflow)?;l.writes+=1;l.bytes=l.bytes.saturating_add((p.len()-4)as u32);let mut merged=false;if l.region_count>0{let idx=l.region_count as usize-1;if let Some(mut r)=l.regions[idx]{if a>=r.start&&a<=r.end{if e>r.end{r.end=e}l.regions[idx]=Some(r);merged=true}}}if !merged{if l.region_count as usize>=l.regions.len(){return Err(PatchLayoutError::TooManyRegions)}l.regions[l.region_count as usize]=Some(PatchRegion{start:a,end:e});l.region_count+=1}}HCI_LAUNCH_RAM=>{if p.len()!=4{return Err(PatchLayoutError::Malformed)}let a=u32::from_le_bytes([p[0],p[1],p[2],p[3]]);if a==u32::MAX{l.sentinel_launches=l.sentinel_launches.saturating_add(1)}else{l.explicit_launches=l.explicit_launches.saturating_add(1)}}x=>return Err(PatchLayoutError::UnsupportedOpcode(x))}}Ok(l)}
#[cfg(test)]mod stage12_tests{use super::*;#[test]fn layout(){let h=[0x4c,0xfc,6,0x00,0x04,0x16,0x00,1,2,0x4c,0xfc,5,0x02,0x04,0x16,0x00,3,0x4c,0xfc,5,0x00,0x00,0x24,0x00,4,0x4e,0xfc,4,0xff,0xff,0xff,0xff];let l=analyze_patch_layout(&h).unwrap();assert_eq!(l.writes,3);assert_eq!(l.region_count,2);assert_eq!(l.regions[0],Some(PatchRegion{start:0x160400,end:0x160403}));assert_eq!(l.sentinel_launches,1);}}

/// Stage 13: exact structural profile of the AP6275P BCM4362A2 PatchRAM reference.
pub const AP6275P_PATCH_SHA256:&str="3e4a1eddaf80f3e45f99e9c77b3cd84c85f605540da5f4f92300b80bca6d67ec";
pub const AP6275P_PATCH_WRITES:u32=462;pub const AP6275P_PATCH_BYTES:u32=69_895;pub const AP6275P_PATCH_SENTINEL_LAUNCHES:u8=1;
pub const AP6275P_PATCH_REGIONS:[PatchRegion;3]=[PatchRegion{start:0x0016_0400,end:0x0016_E7A8},PatchRegion{start:0x0022_1D9C,end:0x0022_24CC},PatchRegion{start:0x0024_0000,end:0x0024_262F}];
pub fn is_ap6275p_patch_layout(l:&PatchLayout)->bool{l.region_count==3&&l.writes==AP6275P_PATCH_WRITES&&l.bytes==AP6275P_PATCH_BYTES&&l.sentinel_launches==1&&l.explicit_launches==0&&l.regions[0]==Some(AP6275P_PATCH_REGIONS[0])&&l.regions[1]==Some(AP6275P_PATCH_REGIONS[1])&&l.regions[2]==Some(AP6275P_PATCH_REGIONS[2])}
#[cfg(test)]mod stage13_tests{use super::*;#[test]fn profile(){let l=PatchLayout{regions:[Some(AP6275P_PATCH_REGIONS[0]),Some(AP6275P_PATCH_REGIONS[1]),Some(AP6275P_PATCH_REGIONS[2])],region_count:3,writes:462,bytes:69_895,sentinel_launches:1,explicit_launches:0};assert!(is_ap6275p_patch_layout(&l));assert_eq!(AP6275P_PATCH_REGIONS.iter().map(|r|r.end-r.start).sum::<u32>(),69_895);}}

/// Stage 14: profile-gated PatchRAM application. This prevents accidentally
/// treating another BCM4362A2 HCD as the AP6275P image recovered here.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Ap6275pApplyError<E>{Layout(PatchLayoutError),WrongProfile,Apply(PatchApplyError<E>)}
pub fn apply_ap6275p_hcd<S:PatchRamSink>(data:&[u8],sink:&mut S)->Result<PatchApplyReport,Ap6275pApplyError<S::Error>>{
    match validate_patch_profile_unordered(data,&LEGACY_AP6275P_PATCH_PROFILE){
        Ok(_)=>{},
        Err(PatchProfileError::Layout(e))=>return Err(Ap6275pApplyError::Layout(e)),
        Err(_)=>return Err(Ap6275pApplyError::WrongProfile),
    }
    apply_hcd(data,sink).map_err(Ap6275pApplyError::Apply)
}
#[cfg(test)]mod stage14_tests{use super::*;struct S;impl PatchRamSink for S{type Error=();fn write_ram(&mut self,_:u32,_:&[u8])->Result<(),()>{Ok(())}fn launch_ram(&mut self,_:LaunchAddress)->Result<(),()>{Ok(())}}#[test]fn rejects_other_profile(){let h=[0x4c,0xfc,5,0,0,0x24,0,1,0x4e,0xfc,4,0xff,0xff,0xff,0xff];let mut s=S;assert_eq!(apply_ap6275p_hcd(&h,&mut s),Err(Ap6275pApplyError::WrongProfile));}}

/// Stage 15: validated AP6275P program object. Construction performs the exact
/// structural profile check once; applying it then cannot target another HCD image.
#[derive(Clone,Copy,Debug)]pub struct Ap6275pPatchProgram<'a>{data:&'a[u8],layout:PatchLayout}
impl<'a> Ap6275pPatchProgram<'a>{pub fn parse(data:&'a[u8])->Result<Self,PatchLayoutError>{validate_patch_profile_unordered(data,&LEGACY_AP6275P_PATCH_PROFILE).map_err(|e|match e{PatchProfileError::Layout(x)=>x,_=>PatchLayoutError::Malformed})?;let p=&LEGACY_AP6275P_PATCH_PROFILE;let layout=PatchLayout{regions:[Some(p.regions[0]),Some(p.regions[1]),Some(p.regions[2])],region_count:3,writes:p.writes,bytes:p.bytes,sentinel_launches:p.sentinel_launches,explicit_launches:p.explicit_launches};Ok(Self{data,layout})}pub const fn layout(&self)->PatchLayout{self.layout}pub fn apply<S:PatchRamSink>(&self,sink:&mut S)->Result<PatchApplyReport,PatchApplyError<S::Error>>{apply_hcd(self.data,sink)}}
#[cfg(test)]mod stage15_tests{use super::*;#[test]fn bad_program_rejected(){let h=[0x4c,0xfc,5,0,0,0x24,0,1,0x4e,0xfc,4,0xff,0xff,0xff,0xff];assert_eq!(Ap6275pPatchProgram::parse(&h).err(),Some(PatchLayoutError::Malformed));}}

/// Stage 16: classify every validated AP6275P WRITE_RAM record by the exact
/// recovered destination region. This deliberately describes PatchRAM layout,
/// not yet the semantic purpose of individual patched functions.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Ap6275pWriteRegion{Code,Data,Config}
pub const fn classify_ap6275p_write(address:u32,bytes:u32)->Option<Ap6275pWriteRegion>{if bytes==0{return None}let end=match address.checked_add(bytes){Some(v)=>v,None=>return None};if address>=PATCH_CODE_START&&end<=PATCH_CODE_END{Some(Ap6275pWriteRegion::Code)}else if address>=PATCH_DATA_START&&end<=PATCH_DATA_END{Some(Ap6275pWriteRegion::Data)}else if address>=PATCH_CONFIG_START&&end<=PATCH_CONFIG_END{Some(Ap6275pWriteRegion::Config)}else{None}}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Ap6275pRegionWriteCounts{pub code_writes:u32,pub data_writes:u32,pub config_writes:u32,pub code_bytes:u32,pub data_bytes:u32,pub config_bytes:u32}
impl Ap6275pPatchProgram<'_>{pub fn for_each_classified_write<F>(&self,mut f:F)->Result<Ap6275pRegionWriteCounts,PatchLayoutError>where F:FnMut(Ap6275pWriteRegion,u32,&[u8]){let mut off=0usize;let mut out=Ap6275pRegionWriteCounts::default();while off<self.data.len(){if self.data.len()-off<3{return Err(PatchLayoutError::Truncated)}let op=u16::from_le_bytes([self.data[off],self.data[off+1]]);let n=self.data[off+2]as usize;off+=3;if self.data.len()-off<n{return Err(PatchLayoutError::Truncated)}let p=&self.data[off..off+n];off+=n;match op{HCI_WRITE_RAM=>{if p.len()<4{return Err(PatchLayoutError::Malformed)}let a=u32::from_le_bytes([p[0],p[1],p[2],p[3]]);let d=&p[4..];let r=classify_ap6275p_write(a,d.len()as u32).ok_or(PatchLayoutError::Malformed)?;match r{Ap6275pWriteRegion::Code=>{out.code_writes+=1;out.code_bytes+=d.len()as u32},Ap6275pWriteRegion::Data=>{out.data_writes+=1;out.data_bytes+=d.len()as u32},Ap6275pWriteRegion::Config=>{out.config_writes+=1;out.config_bytes+=d.len()as u32}}f(r,a,d)},HCI_LAUNCH_RAM=>{if p.len()!=4{return Err(PatchLayoutError::Malformed)}},x=>return Err(PatchLayoutError::UnsupportedOpcode(x))}}Ok(out)}}
#[cfg(test)]mod stage16_tests{use super::*;#[test]fn regions(){assert_eq!(classify_ap6275p_write(PATCH_CODE_START,4),Some(Ap6275pWriteRegion::Code));assert_eq!(classify_ap6275p_write(PATCH_DATA_START,4),Some(Ap6275pWriteRegion::Data));assert_eq!(classify_ap6275p_write(PATCH_CONFIG_START,4),Some(Ap6275pWriteRegion::Config));assert_eq!(classify_ap6275p_write(PATCH_CODE_END-2,4),None);assert_eq!((PATCH_CODE_END-PATCH_CODE_START)+(PATCH_DATA_END-PATCH_DATA_START)+(PATCH_CONFIG_END-PATCH_CONFIG_START),AP6275P_PATCH_BYTES);}}

/// Stage 17: exact AP6275P PatchRAM coverage profile recovered from all 462
/// WRITE_RAM records. Each of the three destination regions is contiguous.
pub const AP6275P_CODE_WRITES:u32=414;pub const AP6275P_DATA_WRITES:u32=8;pub const AP6275P_CONFIG_WRITES:u32=40;
pub const AP6275P_CODE_BYTES:u32=58_280;pub const AP6275P_DATA_BYTES:u32=1_840;pub const AP6275P_CONFIG_BYTES:u32=9_775;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum PatchProfileError{Layout(PatchLayoutError),NonContiguous{region:Ap6275pWriteRegion,expected:u32,actual:u32},UnexpectedProfile,NonSentinelLaunch}
pub fn validate_ap6275p_contiguous_profile(data:&[u8])->Result<Ap6275pRegionWriteCounts,PatchProfileError>{
    validate_patch_profile_unordered(data,&LEGACY_AP6275P_PATCH_PROFILE)
}
#[cfg(test)]mod stage17_tests{use super::*;#[test]fn constants_match_regions(){assert_eq!(AP6275P_CODE_BYTES,PATCH_CODE_END-PATCH_CODE_START);assert_eq!(AP6275P_DATA_BYTES,PATCH_DATA_END-PATCH_DATA_START);assert_eq!(AP6275P_CONFIG_BYTES,PATCH_CONFIG_END-PATCH_CONFIG_START);assert_eq!(AP6275P_CODE_WRITES+AP6275P_DATA_WRITES+AP6275P_CONFIG_WRITES,462);assert_eq!(AP6275P_CODE_BYTES+AP6275P_DATA_BYTES+AP6275P_CONFIG_BYTES,69_895);}}

/// Stage 18: reference identity correction and order-independent structural
/// validation. The HCD command stream is not ordered by destination address;
/// contiguity is therefore proved after sorting WRITE_RAM extents.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct PatchReferenceProfile {
    pub sha256: &'static str,
    pub hcd_size: u32,
    pub writes: u32,
    pub bytes: u32,
    pub sentinel_launches: u8,
    pub explicit_launches: u8,
    pub region_writes: [u32;3],
    pub regions: [PatchRegion;3],
}

pub const LEGACY_AP6275P_PATCH_PROFILE: PatchReferenceProfile = PatchReferenceProfile {
    sha256: "3e4a1eddaf80f3e45f99e9c77b3cd84c85f605540da5f4f92300b80bca6d67ec",
    hcd_size: 73_136,
    writes: 462,
    bytes: 69_895,
    sentinel_launches: 1,
    explicit_launches: 0,
    region_writes: [414,8,40],
    regions: [
        PatchRegion{start:0x0016_0400,end:0x0016_E7A8},
        PatchRegion{start:0x0022_1D9C,end:0x0022_24CC},
        PatchRegion{start:0x0024_0000,end:0x0024_262F},
    ],
};

pub const CURRENT_ORANGEPI_PATCH_PROFILE: PatchReferenceProfile = PatchReferenceProfile {
    sha256: "f7adf14413063f14b0204684fb67ddcd2ae6bca3120343cb9a5cab86a1a545c3",
    hcd_size: 91_900,
    writes: 575,
    bytes: 87_868,
    sentinel_launches: 1,
    explicit_launches: 0,
    region_writes: [518,9,48],
    regions: [
        PatchRegion{start:0x0016_0800,end:0x0017_298C},
        PatchRegion{start:0x0022_1D9C,end:0x0022_2578},
        PatchRegion{start:0x0024_0000,end:0x0024_2DD4},
    ],
};

const MAX_STAGE18_WRITES: usize = 575;
#[derive(Clone,Copy)]
struct PatchExtent { region:u8, start:u32, end:u32 }
const EMPTY_EXTENT: PatchExtent = PatchExtent{region:0,start:0,end:0};

fn classify_profile_write(profile:&PatchReferenceProfile,address:u32,bytes:u32)->Option<u8>{
    if bytes==0{return None}
    let end=address.checked_add(bytes)?;
    let mut i=0usize;
    while i<profile.regions.len(){let r=profile.regions[i];if address>=r.start&&end<=r.end{return Some(i as u8)}i+=1}
    None
}

/// Validates exact HCD structural coverage independent of WRITE_RAM record order.
/// SHA-256 remains a provenance property checked by the host-side runner.
pub fn validate_patch_profile_unordered(data:&[u8],profile:&PatchReferenceProfile)->Result<Ap6275pRegionWriteCounts,PatchProfileError>{
    if data.len()!=profile.hcd_size as usize{return Err(PatchProfileError::UnexpectedProfile)}
    let mut extents=[EMPTY_EXTENT;MAX_STAGE18_WRITES];
    let mut extent_count=0usize;
    let mut off=0usize;
    let mut writes=0u32;let mut bytes=0u32;let mut sentinel=0u8;let mut explicit=0u8;
    let mut counts=Ap6275pRegionWriteCounts::default();
    while off<data.len(){
        if data.len()-off<3{return Err(PatchProfileError::Layout(PatchLayoutError::Truncated))}
        let op=u16::from_le_bytes([data[off],data[off+1]]);let n=data[off+2]as usize;off+=3;
        if data.len()-off<n{return Err(PatchProfileError::Layout(PatchLayoutError::Truncated))}
        let p=&data[off..off+n];off+=n;
        match op{
            HCI_WRITE_RAM=>{
                if p.len()<4{return Err(PatchProfileError::Layout(PatchLayoutError::Malformed))}
                if extent_count>=MAX_STAGE18_WRITES{return Err(PatchProfileError::UnexpectedProfile)}
                let a=u32::from_le_bytes([p[0],p[1],p[2],p[3]]);let dlen=(p.len()-4)as u32;
                let ridx=classify_profile_write(profile,a,dlen).ok_or(PatchProfileError::UnexpectedProfile)?;
                let end=a.checked_add(dlen).ok_or(PatchProfileError::Layout(PatchLayoutError::AddressOverflow))?;
                extents[extent_count]=PatchExtent{region:ridx,start:a,end};extent_count+=1;writes+=1;bytes=bytes.saturating_add(dlen);
                match ridx{0=>{counts.code_writes+=1;counts.code_bytes+=dlen},1=>{counts.data_writes+=1;counts.data_bytes+=dlen},2=>{counts.config_writes+=1;counts.config_bytes+=dlen},_=>return Err(PatchProfileError::UnexpectedProfile)}
            }
            HCI_LAUNCH_RAM=>{
                if p.len()!=4{return Err(PatchProfileError::Layout(PatchLayoutError::Malformed))}
                let a=u32::from_le_bytes([p[0],p[1],p[2],p[3]]);if a==u32::MAX{sentinel=sentinel.saturating_add(1)}else{explicit=explicit.saturating_add(1)}
            }
            x=>return Err(PatchProfileError::Layout(PatchLayoutError::UnsupportedOpcode(x))),
        }
    }
    if writes!=profile.writes||bytes!=profile.bytes||sentinel!=profile.sentinel_launches||explicit!=profile.explicit_launches{return Err(PatchProfileError::UnexpectedProfile)}
    let got_writes=[counts.code_writes,counts.data_writes,counts.config_writes];if got_writes!=profile.region_writes{return Err(PatchProfileError::UnexpectedProfile)}
    let mut i=1usize;while i<extent_count{let key=extents[i];let mut j=i;while j>0{let prev=extents[j-1];if (prev.region,prev.start)<=(key.region,key.start){break}extents[j]=prev;j-=1}extents[j]=key;i+=1}
    let mut cursor=[profile.regions[0].start,profile.regions[1].start,profile.regions[2].start];
    let mut k=0usize;while k<extent_count{let e=extents[k];let r=e.region as usize;if e.start!=cursor[r]{return Err(PatchProfileError::NonContiguous{region:match r{0=>Ap6275pWriteRegion::Code,1=>Ap6275pWriteRegion::Data,_=>Ap6275pWriteRegion::Config},expected:cursor[r],actual:e.start})}cursor[r]=e.end;k+=1}
    if cursor[0]!=profile.regions[0].end||cursor[1]!=profile.regions[1].end||cursor[2]!=profile.regions[2].end{return Err(PatchProfileError::UnexpectedProfile)}
    Ok(counts)
}

/// Structural identity gate for the current Orange Pi HCD. This does not
/// transfer legacy semantic names to current addresses.
pub fn validate_current_orangepi_patch_profile(data:&[u8])->Result<Ap6275pRegionWriteCounts,PatchProfileError>{
    validate_patch_profile_unordered(data,&CURRENT_ORANGEPI_PATCH_PROFILE)
}

#[cfg(test)]
mod stage18_tests {
    use super::*;
    use std::vec::Vec;
    fn wr(h:&mut Vec<u8>,a:u32,d:&[u8]){h.extend_from_slice(&HCI_WRITE_RAM.to_le_bytes());h.push((4+d.len())as u8);h.extend_from_slice(&a.to_le_bytes());h.extend_from_slice(d)}
    #[test]
    fn current_profile_constants(){
        assert_eq!(CURRENT_ORANGEPI_PATCH_PROFILE.region_writes,[518,9,48]);
        assert_eq!(CURRENT_ORANGEPI_PATCH_PROFILE.writes,575);
        assert_eq!(CURRENT_ORANGEPI_PATCH_PROFILE.bytes,87_868);
        assert_eq!(CURRENT_ORANGEPI_PATCH_PROFILE.regions.iter().map(|r|r.end-r.start).sum::<u32>(),87_868);
        assert_ne!(CURRENT_ORANGEPI_PATCH_PROFILE.sha256,LEGACY_AP6275P_PATCH_PROFILE.sha256);
    }
    #[test]
    fn unordered_records_validate(){
        let mut h=Vec::new();wr(&mut h,0x1002,&[3,4]);wr(&mut h,0x3000,&[7,8]);wr(&mut h,0x1000,&[1,2]);wr(&mut h,0x2000,&[5,6]);h.extend_from_slice(&HCI_LAUNCH_RAM.to_le_bytes());h.push(4);h.extend_from_slice(&u32::MAX.to_le_bytes());
        let p=PatchReferenceProfile{sha256:"test",hcd_size:h.len()as u32,writes:4,bytes:8,sentinel_launches:1,explicit_launches:0,region_writes:[2,1,1],regions:[PatchRegion{start:0x1000,end:0x1004},PatchRegion{start:0x2000,end:0x2002},PatchRegion{start:0x3000,end:0x3002}]};
        assert!(validate_patch_profile_unordered(&h,&p).is_ok());
    }
}

/// Stage 19: exact-byte relocation anchors from the legacy AP6275P PatchRAM
/// program into the current Orange Pi HCD.  Full function bytes are identical
/// at each pair below; the addresses are therefore safe structural/semantic
/// anchors.  PC-relative targets still belong to the current image and must be
/// followed there rather than copied from legacy absolute addresses.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ExactPatchFunctionRelocation {
    pub legacy_address:u32,
    pub current_address:u32,
    pub byte_len:u16,
    pub name:&'static str,
}
pub const STAGE19_BT_EXACT_RELOCATIONS:&[ExactPatchFunctionRelocation]=&[
    ExactPatchFunctionRelocation{legacy_address:0x0016_0800,current_address:0x0016_0800,byte_len:368,name:"stage19_patch_anchor_160800"},
    ExactPatchFunctionRelocation{legacy_address:0x0016_2F4C,current_address:0x0016_3668,byte_len:12,name:"bt_index_stride20"},
    ExactPatchFunctionRelocation{legacy_address:0x0016_9104,current_address:0x0016_B7A4,byte_len:10,name:"bt_set_global_60"},
    ExactPatchFunctionRelocation{legacy_address:0x0016_CC80,current_address:0x0017_0178,byte_len:10,name:"bt_u32_gt_2"},
    ExactPatchFunctionRelocation{legacy_address:0x0016_E010,current_address:0x0017_20C0,byte_len:24,name:"bt_find_first_slot_state1"},
];
pub const STAGE19_BT_FUNCTIONS_GE8_COMPARED:u32=266;
pub const STAGE19_BT_FUNCTIONS_GE8_UNIQUE_EXACT:u32=44;
pub const STAGE19_BT_FUNCTIONS_GE8_MULTI_EXACT:u32=9;
pub const STAGE19_BT_FUNCTIONS_GE8_NO_EXACT:u32=213;
pub fn current_patch_address_for_legacy(legacy:u32)->Option<u32>{
    for r in STAGE19_BT_EXACT_RELOCATIONS{if r.legacy_address==legacy{return Some(r.current_address)}}
    None
}
#[cfg(test)]
mod stage19_tests {
    use super::*;
    #[test]fn exact_patch_relocations(){
        assert_eq!(current_patch_address_for_legacy(0x0016_2F4C),Some(0x0016_3668));
        assert_eq!(current_patch_address_for_legacy(0x0016_9104),Some(0x0016_B7A4));
        assert_eq!(current_patch_address_for_legacy(0x0016_CC80),Some(0x0017_0178));
        assert_eq!(current_patch_address_for_legacy(0x0016_E010),Some(0x0017_20C0));
        assert_eq!(STAGE19_BT_FUNCTIONS_GE8_UNIQUE_EXACT+STAGE19_BT_FUNCTIONS_GE8_MULTI_EXACT+STAGE19_BT_FUNCTIONS_GE8_NO_EXACT,STAGE19_BT_FUNCTIONS_GE8_COMPARED);
    }
}

/// Stage 20: current Thumb control-flow anchors derived directly from current
/// bytes inside Stage-19 exact full-body relocations. These are destination
/// addresses, not claims that the destination implementation is byte-identical.
pub const STAGE20_BT_MONOTONIC_UNIQUE_SPINE:u32=44;
pub const STAGE20_BT_CONTEXT_DISAMBIGUATED_EXACT:u32=3;
pub const STAGE20_BT_FIND_FIRST_SLOT_CALLSITE:u32=0x0017_20C6;
pub const STAGE20_BT_FIND_FIRST_SLOT_HELPER:u32=0x0017_2044;
pub const STAGE20_BT_ANCHOR_160800_DIRECT_BL_CALLS:u32=19;
pub const STAGE20_BT_ANCHOR_160800_ROM_TARGETS:&[u32]=&[
    0x0002_02C0,
    0x0002_02E8,
    0x0002_1F0C,
    0x0002_22A4,
    0x0002_24A8,
    0x0004_3D2C,
    0x0004_43FC,
    0x0004_5DD4,
];
pub const fn stage20_bt_is_anchor_rom_target(address:u32)->bool{
    matches!(address,0x0002_02C0|0x0002_02E8|0x0002_1F0C|0x0002_22A4|0x0002_24A8|0x0004_3D2C|0x0004_43FC|0x0004_5DD4)
}
#[cfg(test)]
mod stage20_tests {
    use super::*;
    #[test]
    fn current_control_flow_anchors(){
        assert_eq!(STAGE20_BT_FIND_FIRST_SLOT_CALLSITE,0x0017_20C6);
        assert_eq!(STAGE20_BT_FIND_FIRST_SLOT_HELPER,0x0017_2044);
        assert_eq!(STAGE20_BT_ANCHOR_160800_DIRECT_BL_CALLS,19);
        assert_eq!(STAGE20_BT_ANCHOR_160800_ROM_TARGETS.len(),8);
        assert!(stage20_bt_is_anchor_rom_target(0x0004_3D2C));
        assert!(!stage20_bt_is_anchor_rom_target(0x0017_2044));
        assert_eq!(STAGE20_BT_CONTEXT_DISAMBIGUATED_EXACT,3);
    }
}

/// Stage 21: the Stage-20 helper destination is relocation-normalized identical
/// to legacy sub_16DF94 across all 62 code bytes after canonicalizing only its
/// three direct BL immediates.  The literal pool is checked independently:
/// stack-guard/context word 0x200890 is unchanged and the 7-byte record table
/// moved from 0x222e42 to 0x222fd6.
pub const STAGE21_BT_SLOT_HELPER_LEGACY_ADDR:u32=0x0016_DF94;
pub const STAGE21_BT_SLOT_HELPER_CURRENT_ADDR:u32=0x0017_2044;
pub const STAGE21_BT_SLOT_HELPER_BYTES:u16=62;
pub const STAGE21_BT_SLOT_HELPER_STACK_WORD_ADDR:u32=0x0020_0890;
pub const STAGE21_BT_SLOT_RECORD_BASE:u32=0x0022_2FD6;
pub const STAGE21_BT_SLOT_RECORD_WIDTH:u32=7;
pub const STAGE21_BT_SLOT_COUNT:u8=8;
pub const STAGE21_BT_SLOT_HELPER_ROM_TARGETS:&[u32]=&[0x0000_3D24,0x000F_8CAC,0x0000_94C0];
pub const fn current_bt_slot_record_address(index:u8)->Option<u32>{
    if index<STAGE21_BT_SLOT_COUNT{
        Some(STAGE21_BT_SLOT_RECORD_BASE+(index as u32)*STAGE21_BT_SLOT_RECORD_WIDTH)
    }else{None}
}
#[cfg(test)]
mod stage21_tests {
    use super::*;
    #[test]
    fn relocated_slot_helper(){
        assert_eq!(STAGE21_BT_SLOT_HELPER_CURRENT_ADDR,STAGE20_BT_FIND_FIRST_SLOT_HELPER);
        assert_eq!(STAGE21_BT_SLOT_HELPER_BYTES,62);
        assert_eq!(current_bt_slot_record_address(0),Some(0x0022_2FD6));
        assert_eq!(current_bt_slot_record_address(7),Some(0x0022_3007));
        assert_eq!(current_bt_slot_record_address(8),None);
        assert_eq!(STAGE21_BT_SLOT_HELPER_ROM_TARGETS,&[0x0000_3D24,0x000F_8CAC,0x0000_94C0]);
    }
}

/// Stage 22: current slot-helper semantics recovered by combining the Stage-21
/// relocation-normalized body with its stable ROM call roles. 0x3D24 and
/// 0xF8CAC were already behaviorally recovered as memset/memcmp; 0x94C0 is the
/// stack-canary terminal sink. The current helper retains those exact targets.
pub const STAGE22_CURRENT_BT_MEMSET_ADDR:u32=ROM_MEMSET_ADDR;
pub const STAGE22_CURRENT_BT_MEMCMP_ADDR:u32=ROM_MEMCMP_ADDR;
pub const STAGE22_CURRENT_BT_STACK_GUARD_FAIL_ADDR:u32=ROM_STACK_GUARD_FAIL_ADDR;
pub const STAGE22_BT_SLOT_RECORD_BYTES:usize=7;
pub const STAGE22_BT_SLOT_COUNT:usize=8;
pub fn bt_slot_record_is_empty(record:&[u8;STAGE22_BT_SLOT_RECORD_BYTES])->bool{
    let mut i=0usize;while i<record.len(){if record[i]!=0{return false}i+=1}true
}
/// Semantic replacement for current `bt_find_first_slot_state1`: the helper's
/// state value 1 is precisely the "all seven record bytes are zero" condition.
pub fn bt_find_first_empty_slot(records:&[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT])->u8{
    let mut i=0usize;while i<records.len(){if bt_slot_record_is_empty(&records[i]){return i as u8}i+=1}STAGE22_BT_SLOT_COUNT as u8
}
#[cfg(test)]
mod stage22_tests{
    use super::*;
    #[test]fn empty_slot_semantics(){
        assert_eq!(STAGE22_CURRENT_BT_MEMSET_ADDR,0x3D24);
        assert_eq!(STAGE22_CURRENT_BT_MEMCMP_ADDR,0xF8CAC);
        assert_eq!(STAGE22_CURRENT_BT_STACK_GUARD_FAIL_ADDR,0x94C0);
        let mut r=[[1u8;7];8];r[3]=[0;7];
        assert!(bt_slot_record_is_empty(&r[3]));assert!(!bt_slot_record_is_empty(&r[0]));
        assert_eq!(bt_find_first_empty_slot(&r),3);
        r[3]=[1;7];assert_eq!(bt_find_first_empty_slot(&r),8);
    }
}

/// Stage 23: current slot-table lifecycle functions. Each address below is the
/// unique relocation-normalized match of the complete legacy function body;
/// current literal words and direct branch targets are independently checked
/// by the Stage-23 runner before this source is applied.
pub const STAGE23_CURRENT_BT_FIND_MATCHING_SLOT_ADDR:u32=0x0017_208C;
pub const STAGE23_CURRENT_BT_CLEAR_SLOT_TABLE_ADDR:u32=0x0017_23C0;
pub const STAGE23_CURRENT_BT_INSERT_SLOT_IF_ABSENT_ADDR:u32=0x0017_23D0;
pub const STAGE23_CURRENT_BT_REMOVE_SLOT_ADDR:u32=0x0017_2458;
pub const STAGE23_CURRENT_BT_RESET_TABLE_TAIL_ADDR:u32=0x0017_2480;
pub const STAGE23_CURRENT_BT_MEMCPY_ADDR:u32=ROM_MEMCPY_ADDR;
pub const STAGE23_BT_SLOT_PAYLOAD_BYTES:usize=6;
pub const STAGE23_BT_TABLE_BYTES:usize=STAGE22_BT_SLOT_RECORD_BYTES*STAGE22_BT_SLOT_COUNT;

/// Semantic replacement for current `sub_16DFDC`: find the first record whose
/// tag byte and six-byte payload both match, or return 8 when absent.
pub fn bt_find_matching_slot(
    records:&[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
    tag:u8,
    payload:&[u8;STAGE23_BT_SLOT_PAYLOAD_BYTES],
)->u8{
    let mut i=0usize;
    while i<records.len(){
        let r=&records[i];
        if r[0]==tag{
            let mut j=0usize;let mut same=true;
            while j<STAGE23_BT_SLOT_PAYLOAD_BYTES{
                if r[j+1]!=payload[j]{same=false;break}
                j+=1;
            }
            if same{return i as u8}
        }
        i+=1;
    }
    STAGE22_BT_SLOT_COUNT as u8
}

/// Semantic replacement for current `sub_16E2B0`.
pub fn bt_clear_slot_table(records:&mut[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT]){
    let mut i=0usize;
    while i<records.len(){records[i]=[0;STAGE22_BT_SLOT_RECORD_BYTES];i+=1}
}

/// Semantic replacement for the table part of current `sub_16E2C0`.
/// Return values preserve the firmware: 0 for duplicate or successful insert,
/// and 17 when no empty slot exists.
pub fn bt_insert_slot_if_absent(
    records:&mut[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
    tag:u8,
    payload:&[u8;STAGE23_BT_SLOT_PAYLOAD_BYTES],
)->u8{
    if bt_find_matching_slot(records,tag,payload)<STAGE22_BT_SLOT_COUNT as u8{return 0}
    let slot=bt_find_first_empty_slot(records);
    if slot>=STAGE22_BT_SLOT_COUNT as u8{return 17}
    let r=&mut records[slot as usize];r[0]=tag;
    let mut j=0usize;while j<STAGE23_BT_SLOT_PAYLOAD_BYTES{r[j+1]=payload[j];j+=1}
    0
}

/// Semantic replacement for current `sub_16E348`: clear a matching record and
/// return 1, or return 0 when the key/payload pair is absent.
pub fn bt_remove_slot(
    records:&mut[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
    tag:u8,
    payload:&[u8;STAGE23_BT_SLOT_PAYLOAD_BYTES],
)->u8{
    let slot=bt_find_matching_slot(records,tag,payload);
    if slot>=STAGE22_BT_SLOT_COUNT as u8{return 0}
    records[slot as usize]=[0;STAGE22_BT_SLOT_RECORD_BYTES];1
}
#[cfg(test)]
mod stage23_tests{
    use super::*;
    #[test]fn slot_table_lifecycle(){
        assert_eq!(STAGE23_BT_TABLE_BYTES,56);
        assert_eq!(STAGE23_CURRENT_BT_MEMCPY_ADDR,0x3DB4);
        let p1=[1,2,3,4,5,6];let p2=[6,5,4,3,2,1];
        let mut r=[[0u8;7];8];
        assert_eq!(bt_find_matching_slot(&r,9,&p1),8);
        assert_eq!(bt_insert_slot_if_absent(&mut r,9,&p1),0);
        assert_eq!(r[0],[9,1,2,3,4,5,6]);
        assert_eq!(bt_insert_slot_if_absent(&mut r,9,&p1),0);
        assert_eq!(bt_find_matching_slot(&r,9,&p1),0);
        assert_eq!(bt_remove_slot(&mut r,9,&p1),1);assert_eq!(r[0],[0;7]);
        assert_eq!(bt_remove_slot(&mut r,9,&p1),0);
        let mut i=0u8;while (i as usize)<r.len(){let p=[i,1,2,3,4,5];assert_eq!(bt_insert_slot_if_absent(&mut r,i+1,&p),0);i+=1}
        assert_eq!(bt_insert_slot_if_absent(&mut r,99,&p2),17);
        bt_clear_slot_table(&mut r);assert_eq!(r,[[0;7];8]);
        // Exact firmware quirk: an all-zero key/payload already matches an empty record.
        assert_eq!(bt_find_matching_slot(&r,0,&[0;6]),0);
    }
    #[test]fn current_consumer_addresses(){
        assert_eq!(STAGE23_CURRENT_BT_FIND_MATCHING_SLOT_ADDR,0x17208C);
        assert_eq!(STAGE23_CURRENT_BT_CLEAR_SLOT_TABLE_ADDR,0x1723C0);
        assert_eq!(STAGE23_CURRENT_BT_INSERT_SLOT_IF_ABSENT_ADDR,0x1723D0);
        assert_eq!(STAGE23_CURRENT_BT_REMOVE_SLOT_ADDR,0x172458);
        assert_eq!(STAGE23_CURRENT_BT_RESET_TABLE_TAIL_ADDR,0x172480);
    }
}

/// Stage 24: adjacent current Bluetooth control-plane functions proven by
/// unique relocation-normalized complete-body identity plus re-read current
/// literals/direct branch targets.
pub const STAGE24_CURRENT_BT_EVENT_INSERT_ADAPTER_ADDR:u32=0x0017_2408;
pub const STAGE24_CURRENT_BT_EVENT_STATE_DISPATCH_ADDR:u32=0x0017_2430;
pub const STAGE24_CURRENT_BT_RESET_SLOT_SUBSYSTEM_ADDR:u32=0x0017_2480;
pub const STAGE24_CURRENT_BT_REMOVE_TAG0_OR1_ADDR:u32=0x0017_24C8;
pub const STAGE24_CURRENT_BT_PREPARE_CAPPED_PAYLOAD_ADDR:u32=0x0017_24E4;

pub const STAGE24_BT_MODE_ADDR:u32=0x0022_3064;
pub const STAGE24_BT_AUX_FLAG_ADDR:u32=0x0022_2FD0;
pub const STAGE24_BT_RESET_CONTEXT_A:u32=0x0022_304C;
pub const STAGE24_BT_RESET_CONTEXT_B:u32=0x0022_3068;
pub const STAGE24_BT_SCRATCH_ADDR:u32=0x0022_300E;
pub const STAGE24_BT_OPAQUE_RESET_BOUNDARY_ADDR:u32=0x0001_51BC;
pub const STAGE24_BT_SCRATCH_BYTES:usize=59;
pub const STAGE24_BT_SCRATCH_PAYLOAD_MAX:usize=58;

pub trait BtOpaqueResetBoundary{fn call(&mut self,context_address:u32);}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtSlotSubsystemState{
    pub mode:u8,
    pub aux_flag:u8,
    pub records:[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
}

/// Semantic replacement for current `0x172480` table/reset logic. The two calls
/// to 0x151BC stay opaque; current instruction bytes prove only their argument
/// addresses and ordering.
pub fn bt_reset_slot_subsystem<B:BtOpaqueResetBoundary>(state:&mut BtSlotSubsystemState,b:&mut B){
    if state.mode!=0{
        b.call(STAGE24_BT_RESET_CONTEXT_A);
        if state.mode==4{
            b.call(STAGE24_BT_RESET_CONTEXT_B);
            state.aux_flag=0;
        }
        state.mode=0;
    }
    bt_clear_slot_table(&mut state.records);
}

/// Semantic replacement for current `0x1724C8`: remove payload under tag 0;
/// if absent, retry tag 1.
pub fn bt_remove_payload_tag0_or1(
    records:&mut[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
    payload:&[u8;STAGE23_BT_SLOT_PAYLOAD_BYTES],
)->u8{
    let r=bt_remove_slot(records,0,payload);
    if r!=0{return r}
    bt_remove_slot(records,1,payload)
}

/// Semantic replacement for current `0x1724E4` for valid callers that provide
/// at least 58 source bytes. It clears 59 output bytes, stores the low byte of
/// `input_len >> 1` at byte 0, then copies min(input_len,58) bytes to byte 1.
pub fn bt_prepare_capped_payload(
    input_len:u32,
    source:&[u8;STAGE24_BT_SCRATCH_PAYLOAD_MAX],
    out:&mut[u8;STAGE24_BT_SCRATCH_BYTES],
){
    *out=[0;STAGE24_BT_SCRATCH_BYTES];
    out[0]=(input_len>>1) as u8;
    let n=core::cmp::min(input_len as usize,STAGE24_BT_SCRATCH_PAYLOAD_MAX);
    let mut i=0usize;while i<n{out[i+1]=source[i];i+=1}
}

#[cfg(test)]
mod stage24_tests{
    use super::*;
    #[derive(Default)]struct B{n:u8,a:[u32;2]}impl BtOpaqueResetBoundary for B{fn call(&mut self,x:u32){self.a[self.n as usize]=x;self.n+=1}}
    #[test]fn reset_and_dual_tag_remove(){
        assert_eq!(STAGE24_CURRENT_BT_RESET_SLOT_SUBSYSTEM_ADDR,0x172480);
        let mut s=BtSlotSubsystemState{mode:4,aux_flag:7,records:[[1;7];8]};let mut b=B::default();
        bt_reset_slot_subsystem(&mut s,&mut b);
        assert_eq!((s.mode,s.aux_flag,b.n,b.a),(0,0,2,[STAGE24_BT_RESET_CONTEXT_A,STAGE24_BT_RESET_CONTEXT_B]));
        assert_eq!(s.records,[[0;7];8]);
        let p=[1,2,3,4,5,6];let mut r=[[0u8;7];8];
        assert_eq!(bt_insert_slot_if_absent(&mut r,1,&p),0);
        assert_eq!(bt_remove_payload_tag0_or1(&mut r,&p),1);
        assert_eq!(bt_remove_payload_tag0_or1(&mut r,&p),0);
    }
    #[test]fn reset_mode_zero_still_clears_table(){
        let mut s=BtSlotSubsystemState{mode:0,aux_flag:9,records:[[2;7];8]};let mut b=B::default();
        bt_reset_slot_subsystem(&mut s,&mut b);assert_eq!(b.n,0);assert_eq!(s.aux_flag,9);assert_eq!(s.records,[[0;7];8]);
    }
    #[test]fn capped_payload_builder(){
        let mut src=[0u8;58];let mut i=0usize;while i<src.len(){src[i]=i as u8;i+=1}
        let mut out=[0xAAu8;59];bt_prepare_capped_payload(4,&src,&mut out);
        assert_eq!(out[0],2);assert_eq!(&out[1..5],&[0,1,2,3]);assert!(out[5..].iter().all(|&x|x==0));
        bt_prepare_capped_payload(100,&src,&mut out);assert_eq!(out[0],50);assert_eq!(&out[1..],&src);
    }
}
/// Stage 25: current Bluetooth pair configuration and event/control dispatch
/// recovered from unique relocation-normalized bodies plus current literals and
/// direct targets. Unnamed ROM exits remain explicit boundaries/routes.
pub const STAGE25_CURRENT_BT_PAIR_CONFIG_WRITE_ADDR:u32=0x0017_21EC;
pub const STAGE25_CURRENT_BT_PAIR_CONFIG_LOOKUP_ADDR:u32=0x0017_2220;
pub const STAGE25_CURRENT_BT_EVENT_INSERT_ADAPTER_ADDR:u32=0x0017_2408;
pub const STAGE25_CURRENT_BT_EVENT_STATE_DISPATCH_ADDR:u32=0x0017_2430;
pub const STAGE25_CURRENT_BT_MODE_MACHINE_ADDR:u32=0x0017_2518;
pub const STAGE25_CURRENT_BT_COMMAND_DISPATCH_ADDR:u32=0x0017_2594;
pub const STAGE25_CURRENT_BT_MODE_EDGE_DISPATCH_ADDR:u32=0x0017_25DC;
pub const STAGE25_CURRENT_BT_INIT_WRAPPER_ADDR:u32=0x0017_25F8;

pub const STAGE25_BT_PAIR_DIRTY_ADDR:u32=0x0022_2FD0;
pub const STAGE25_BT_COMMAND_BYTE_ADDR:u32=0x0022_2FD1;
pub const STAGE25_BT_PAIR_CONFIG_ADDR:u32=0x0022_2FD2;
pub const STAGE25_BT_PAIR_DEFAULT_ADDR:u32=0x0022_207C;
pub const STAGE25_BT_EVENT_LOOKUP_BOUNDARY_ADDR:u32=0x0008_D34C;
pub const STAGE25_BT_EVENT_OTHER_EXIT_ADDR:u32=0x0006_E4B4;
pub const STAGE25_BT_EVENT_MODE23_EXIT_ADDR:u32=0x0001_1EA8;
pub const STAGE25_BT_MODE1_TARGET_ADDR:u32=0x0017_218C;
pub const STAGE25_BT_MODE2_TARGET_ADDR:u32=0x0017_20E8;

/// Semantic replacement for current `0x1721EC` / legacy `sub_16E13C`.
/// `triple` is selector,key,value. Only selectors 0..1 and nonzero values are
/// accepted. Firmware status 18 is preserved for rejected input.
pub fn bt_pair_config_write(
    pairs:&mut[[u8;2];2],dirty:&mut u8,triple:[u8;3],
)->u8{
    let selector=triple[0] as usize;
    if selector>1 || triple[2]==0{return 18}
    pairs[selector]=[triple[1],triple[2]];
    *dirty=1;0
}

/// Semantic replacement for current `0x172220` / legacy `sub_16E170`.
pub fn bt_pair_config_lookup(dirty:u8,pairs:&[[u8;2];2],fallback:u8,key:u8)->u8{
    if dirty!=0{
        if pairs[0][0]==key{return pairs[0][1]}
        if pairs[1][0]==key{return pairs[1][1]}
    }
    fallback
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtEventLookupRecord{
    pub tag:u8,
    pub payload:[u8;STAGE23_BT_SLOT_PAYLOAD_BYTES],
}
pub trait BtEventObjectLookup{
    fn lookup(&mut self,key:u16)->Option<BtEventLookupRecord>;
}

/// Semantic replacement for current `0x172408`. The integer return preserves
/// the pointer-shaped firmware result: a guard miss returns `event_address`, a
/// lookup miss returns zero, and a found object returns the slot-insert status.
pub fn bt_event_insert_adapter<L:BtEventObjectLookup>(
    event_address:u32,event:&[u8;14],
    records:&mut[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
    lookup:&mut L,
)->u32{
    if event[8]!=8 || event[13]==0{return event_address}
    let key=u16::from_le_bytes([event[11],event[12]]);
    let Some(object)=lookup.lookup(key) else{return 0};
    bt_insert_slot_if_absent(records,object.tag,&object.payload) as u32
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum BtEventStateRoute{OtherMode,Mode2Or3}
pub const fn bt_event_state_route(mode:u8)->BtEventStateRoute{
    if mode==2||mode==3{BtEventStateRoute::Mode2Or3}else{BtEventStateRoute::OtherMode}
}
pub trait BtEventStateExit{
    fn other_mode(&mut self,event_address:u32);
    fn mode_2_or_3(&mut self,event_address:u32);
}

/// Semantic control-flow model of current `0x172430`. For modes other than
/// 2/3 the event adapter runs first, but the tail boundary receives the
/// original event pointer exactly as in the current instruction sequence.
pub fn bt_event_state_dispatch<L:BtEventObjectLookup,E:BtEventStateExit>(
    mode:u8,event_address:u32,event:&[u8;14],
    records:&mut[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
    lookup:&mut L,exit:&mut E,
){
    match bt_event_state_route(mode){
        BtEventStateRoute::Mode2Or3=>exit.mode_2_or_3(event_address),
        BtEventStateRoute::OtherMode=>{
            let _=bt_event_insert_adapter(event_address,event,records,lookup);
            exit.other_mode(event_address);
        }
    }
}

pub trait BtModeMachineBoundary{fn run(&mut self)->u8;}

/// Semantic command switch of current `0x172594`. `frame` is fixed at 59 bytes
/// so opcode 1 can reproduce the firmware's maximum 58-byte tail copy without
/// adding a host-only truncation rule.
pub fn bt_control_command_dispatch<M:BtModeMachineBoundary>(
    frame_len:u8,frame:&[u8;STAGE24_BT_SCRATCH_BYTES],
    scratch:&mut[u8;STAGE24_BT_SCRATCH_BYTES],command_byte:&mut u8,
    pairs:&mut[[u8;2];2],dirty:&mut u8,mode_machine:&mut M,
)->u8{
    match frame[0]{
        1=>{
            let n=frame_len.wrapping_sub(1) as usize;
            *scratch=[0;STAGE24_BT_SCRATCH_BYTES];
            scratch[0]=((n as u32)>>1) as u8;
            let copy=core::cmp::min(n,STAGE24_BT_SCRATCH_PAYLOAD_MAX);
            let mut i=0usize;while i<copy{scratch[i+1]=frame[i+1];i+=1}
            0
        }
        2=>{*command_byte=frame[1];0}
        3=>bt_pair_config_write(pairs,dirty,[frame[1],frame[2],frame[3]]),
        4=>mode_machine.run(),
        _=>18,
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum BtModeEdgeRoute{Mode1,Mode2,None}
pub const fn bt_mode_edge_route(mode:u8)->BtModeEdgeRoute{
    match mode{1=>BtModeEdgeRoute::Mode1,2=>BtModeEdgeRoute::Mode2,_=>BtModeEdgeRoute::None}
}

#[cfg(test)]
mod stage25_tests{
    use super::*;
    struct L{key:u16,record:Option<BtEventLookupRecord>,calls:u8}
    impl BtEventObjectLookup for L{fn lookup(&mut self,k:u16)->Option<BtEventLookupRecord>{self.calls+=1;if k==self.key{self.record}else{None}}}
    #[derive(Default)]struct E{other:u8,m23:u8,last:u32}
    impl BtEventStateExit for E{
        fn other_mode(&mut self,a:u32){self.other+=1;self.last=a}
        fn mode_2_or_3(&mut self,a:u32){self.m23+=1;self.last=a}
    }
    struct M{v:u8,n:u8}impl BtModeMachineBoundary for M{fn run(&mut self)->u8{self.n+=1;self.v}}
    #[test]fn pair_config_semantics(){
        assert_eq!(STAGE25_CURRENT_BT_PAIR_CONFIG_WRITE_ADDR,0x1721EC);
        assert_eq!(STAGE25_CURRENT_BT_PAIR_CONFIG_LOOKUP_ADDR,0x172220);
        let mut p=[[0u8;2];2];let mut d=0u8;
        assert_eq!(bt_pair_config_write(&mut p,&mut d,[0,7,9]),0);
        assert_eq!((p,d),([[7,9],[0,0]],1));
        assert_eq!(bt_pair_config_write(&mut p,&mut d,[2,1,2]),18);
        assert_eq!(bt_pair_config_write(&mut p,&mut d,[1,3,0]),18);
        assert_eq!(bt_pair_config_lookup(d,&p,55,7),9);
        assert_eq!(bt_pair_config_lookup(d,&p,55,8),55);
        assert_eq!(bt_pair_config_lookup(0,&p,55,7),55);
    }
    #[test]fn event_adapter_and_state_route(){
        let rec=BtEventLookupRecord{tag:4,payload:[1,2,3,4,5,6]};
        let mut l=L{key:0x1234,record:Some(rec),calls:0};let mut records=[[0u8;7];8];
        let mut ev=[0u8;14];ev[8]=8;ev[11]=0x34;ev[12]=0x12;ev[13]=1;
        assert_eq!(bt_event_insert_adapter(0x1000,&ev,&mut records,&mut l),0);
        assert_eq!(records[0],[4,1,2,3,4,5,6]);assert_eq!(l.calls,1);
        ev[8]=7;assert_eq!(bt_event_insert_adapter(0x12345678,&ev,&mut records,&mut l),0x12345678);
        let mut e=E::default();ev[8]=8;
        bt_event_state_dispatch(2,9,&ev,&mut records,&mut l,&mut e);assert_eq!((e.other,e.m23,e.last),(0,1,9));
        bt_event_state_dispatch(4,10,&ev,&mut records,&mut l,&mut e);assert_eq!((e.other,e.m23,e.last),(1,1,10));
        assert_eq!(bt_event_state_route(3),BtEventStateRoute::Mode2Or3);
        assert_eq!(bt_event_state_route(255),BtEventStateRoute::OtherMode);
    }
    #[test]fn command_and_mode_edge_dispatch(){
        let mut frame=[0u8;59];let mut scratch=[0xAAu8;59];let mut cmd=0u8;let mut p=[[0u8;2];2];let mut d=0u8;let mut m=M{v:3,n:0};
        frame[0]=1;frame[1]=10;frame[2]=11;frame[3]=12;
        assert_eq!(bt_control_command_dispatch(4,&frame,&mut scratch,&mut cmd,&mut p,&mut d,&mut m),0);
        assert_eq!((scratch[0],scratch[1],scratch[2],scratch[3]),(1,10,11,12));
        frame[0]=2;frame[1]=77;assert_eq!(bt_control_command_dispatch(2,&frame,&mut scratch,&mut cmd,&mut p,&mut d,&mut m),0);assert_eq!(cmd,77);
        frame[0]=3;frame[1]=1;frame[2]=5;frame[3]=6;assert_eq!(bt_control_command_dispatch(4,&frame,&mut scratch,&mut cmd,&mut p,&mut d,&mut m),0);assert_eq!(p[1],[5,6]);
        frame[0]=4;assert_eq!(bt_control_command_dispatch(1,&frame,&mut scratch,&mut cmd,&mut p,&mut d,&mut m),3);assert_eq!(m.n,1);
        frame[0]=9;assert_eq!(bt_control_command_dispatch(1,&frame,&mut scratch,&mut cmd,&mut p,&mut d,&mut m),18);
        assert_eq!(bt_mode_edge_route(1),BtModeEdgeRoute::Mode1);assert_eq!(bt_mode_edge_route(2),BtModeEdgeRoute::Mode2);assert_eq!(bt_mode_edge_route(0),BtModeEdgeRoute::None);
    }
}

/// Stage 26: current Bluetooth mode-machine, init-wrapper, and post-init MMIO
/// program recovered from unique relocation-normalized complete-body identity.
/// Unresolved ROM entries remain explicit boundary traits.
pub const STAGE26_CURRENT_BT_MODE_MACHINE_ADDR:u32=0x0017_2518;
pub const STAGE26_CURRENT_BT_INIT_WRAPPER_ADDR:u32=0x0017_25F8;
pub const STAGE26_CURRENT_BT_POST_INIT_REG_PROGRAM_ADDR:u32=0x0017_294C;

pub const STAGE26_BT_MODE_ADDR:u32=0x0022_3064;
pub const STAGE26_BT_MODE_MIRROR_ADDR:u32=0x0022_3065;
pub const STAGE26_BT_MODE_SOURCE_ADDR:u32=0x0022_2084;
pub const STAGE26_BT_MODE_INTERVAL_ADDR:u32=0x0022_208C;
pub const STAGE26_BT_MODE_CONTEXT_ADDR:u32=0x0022_304C;
pub const STAGE26_BT_MODE_CALLBACK_THUMB:u32=0x0017_1FF9;
pub const STAGE26_BT_MODE_PRELUDE_BOUNDARY:u32=0x0007_2B24;
pub const STAGE26_BT_MODE_REGISTER_BOUNDARY:u32=0x0001_51FE;
pub const STAGE26_BT_MODE_ONE_ARG_BOUNDARY:u32=0x0001_51BC;
pub const STAGE26_BT_MODE_TWO_ARG_BOUNDARY:u32=0x0001_5180;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtModeMachineState{
    pub mode:u8,
    pub source_byte:u8,
    pub mirrored_byte:u8,
    pub interval:u32,
}
pub trait BtModeMachineOpaqueBoundaries{
    fn boundary_72b24(&mut self,arg0:u32);
    fn boundary_151fe(&mut self,context:u32,callback_thumb:u32,zero:u32,interval:u32);
    fn boundary_151bc(&mut self,context:u32);
    fn boundary_15180(&mut self,context:u32,interval:u32);
}

/// Source-level control model of current `0x172518` / legacy `sub_16E408`.
/// The ROM entry points remain unnamed; only exact arguments/order and global
/// state transitions are promoted.
pub fn bt_mode_machine_step<B:BtModeMachineOpaqueBoundaries>(state:&mut BtModeMachineState,b:&mut B)->u8{
    match state.mode{
        0=>{
            b.boundary_72b24(0);
            state.mode=1;
            state.mirrored_byte=state.source_byte;
            b.boundary_151fe(STAGE26_BT_MODE_CONTEXT_ADDR,STAGE26_BT_MODE_CALLBACK_THUMB,0,state.interval);
            b.boundary_15180(STAGE26_BT_MODE_CONTEXT_ADDR,state.interval);
        }
        1=>{
            state.mirrored_byte=state.source_byte;
            b.boundary_151bc(STAGE26_BT_MODE_CONTEXT_ADDR);
            b.boundary_15180(STAGE26_BT_MODE_CONTEXT_ADDR,state.interval);
        }
        2|3|4=>state.mode=5,
        _=>{},
    }
    if state.mode<2{0}else{3}
}

pub const STAGE26_BT_INIT_WORKSPACE_ADDR:u32=0x0021_7C8C;
pub const STAGE26_BT_INIT_WORKSPACE_BYTES:usize=268;
pub const STAGE26_BT_INIT_STATUS_ADDR:u32=0x0032_0180;
pub const STAGE26_BT_INIT_CONTEXT_A:u32=0x0021_7D48;
pub const STAGE26_BT_INIT_CONTEXT_B:u32=0x0021_7D6C;
pub const STAGE26_BT_INIT_WORD_ADDR:u32=0x0020_4BC8;
pub const STAGE26_BT_INIT_CONFIG_ADDR:u32=0x0022_2570;
pub const STAGE26_BT_INIT_CALLBACK_THUMB:u32=0x000B_FC11;
pub const STAGE26_BT_INIT_PROBE_BOUNDARY:u32=0x000B_FAD0;
pub const STAGE26_BT_INIT_APPLY_BOUNDARY:u32=0x000B_F9F4;
pub const STAGE26_BT_INIT_CONTEXT_A_BOUNDARY:u32=0x000B_0864;
pub const STAGE26_BT_INIT_CONTEXT_B_BOUNDARY:u32=0x0001_7670;
pub const STAGE26_BT_INIT_REGISTER_BOUNDARY:u32=0x0001_44BC;

pub trait BtInitOpaqueBoundaries{
    fn probe(&mut self,arg0:u32)->u32;
    fn apply_probe(&mut self,token:u32);
    fn init_context_a(&mut self,address:u32);
    fn init_context_b(&mut self,address:u32);
    fn register(&mut self,workspace:u32,config:u32,kind:u32,callback_thumb:u32,arg4:u32,arg5:u32,word:u16)->u32;
}

/// Source-level sequencing model of current `0x1725F8` / legacy `sub_16E4E8`.
/// The 268-byte memset is libre; the other five ROM calls remain explicit
/// opaque boundaries with their current arguments frozen.
pub fn bt_init_wrapper<B:BtInitOpaqueBoundaries>(
    workspace:&mut[u8;STAGE26_BT_INIT_WORKSPACE_BYTES],status_word:u32,registration_word:u16,b:&mut B,
)->u32{
    *workspace=[0;STAGE26_BT_INIT_WORKSPACE_BYTES];
    let token=b.probe(0);
    if status_word&4==0{b.apply_probe(token);}
    b.init_context_a(STAGE26_BT_INIT_CONTEXT_A);
    b.init_context_b(STAGE26_BT_INIT_CONTEXT_B);
    b.register(
        STAGE26_BT_INIT_WORKSPACE_ADDR,STAGE26_BT_INIT_CONFIG_ADDR,23,
        STAGE26_BT_INIT_CALLBACK_THUMB,0,0,registration_word,
    )
}

pub const STAGE26_BT_MMIO_WRITE10_ADDR:u32=0x0042_3758;
pub const STAGE26_BT_MMIO_OR800_ADDR:u32=0x0064_085C;
pub const STAGE26_BT_MMIO_MASK_F80_ADDR:u32=0x0064_0834;
pub const STAGE26_BT_MMIO_MASK_F8_ADDR:u32=0x0042_0BE0;
pub trait BtMmio32{fn read32(&mut self,address:u32)->u32;fn write32(&mut self,address:u32,value:u32);}

/// Exact register program of current `0x17294C` / legacy `sub_16E768`.
pub fn bt_post_init_register_program<I:BtMmio32>(io:&mut I)->u8{
    io.write32(STAGE26_BT_MMIO_WRITE10_ADDR,10);
    let a=io.read32(STAGE26_BT_MMIO_OR800_ADDR);
    io.write32(STAGE26_BT_MMIO_OR800_ADDR,a|0x800);
    let b=io.read32(STAGE26_BT_MMIO_MASK_F80_ADDR);
    io.write32(STAGE26_BT_MMIO_MASK_F80_ADDR,(b&!0xF80)|0x100);
    let c=io.read32(STAGE26_BT_MMIO_MASK_F8_ADDR);
    io.write32(STAGE26_BT_MMIO_MASK_F8_ADDR,(c&!0xF8)|0xE8);
    0
}

#[cfg(test)]
mod stage26_tests{
    use super::*;
    #[derive(Default)]struct M{calls:[u8;4],n:usize,args:[[u32;4];4]}
    impl BtModeMachineOpaqueBoundaries for M{
        fn boundary_72b24(&mut self,a:u32){self.calls[self.n]=1;self.args[self.n][0]=a;self.n+=1}
        fn boundary_151fe(&mut self,c:u32,cb:u32,z:u32,i:u32){self.calls[self.n]=2;self.args[self.n]=[c,cb,z,i];self.n+=1}
        fn boundary_151bc(&mut self,c:u32){self.calls[self.n]=3;self.args[self.n][0]=c;self.n+=1}
        fn boundary_15180(&mut self,c:u32,i:u32){self.calls[self.n]=4;self.args[self.n][0]=c;self.args[self.n][1]=i;self.n+=1}
    }
    #[test]fn mode_machine_preserves_transitions_and_order(){
        let mut s=BtModeMachineState{mode:0,source_byte:7,mirrored_byte:0,interval:99};let mut m=M::default();
        assert_eq!(bt_mode_machine_step(&mut s,&mut m),0);assert_eq!((s.mode,s.mirrored_byte),(1,7));
        assert_eq!(&m.calls[..m.n],&[1,2,4]);assert_eq!(m.args[1],[STAGE26_BT_MODE_CONTEXT_ADDR,STAGE26_BT_MODE_CALLBACK_THUMB,0,99]);
        m=M::default();s.source_byte=8;assert_eq!(bt_mode_machine_step(&mut s,&mut m),0);assert_eq!(s.mirrored_byte,8);assert_eq!(&m.calls[..m.n],&[3,4]);
        for mode in [2u8,3,4]{let mut x=BtModeMachineState{mode,source_byte:1,mirrored_byte:2,interval:3};let mut q=M::default();assert_eq!(bt_mode_machine_step(&mut x,&mut q),3);assert_eq!(x.mode,5);assert_eq!(q.n,0);}
        let mut x=BtModeMachineState{mode:9,source_byte:1,mirrored_byte:2,interval:3};let mut q=M::default();assert_eq!(bt_mode_machine_step(&mut x,&mut q),3);assert_eq!(x.mode,9);
    }
    #[derive(Default)]struct I{calls:[u8;5],n:usize,last:[u32;7],probe_value:u32,ret:u32}
    impl BtInitOpaqueBoundaries for I{
        fn probe(&mut self,a:u32)->u32{self.calls[self.n]=1;self.n+=1;self.last[0]=a;self.probe_value}
        fn apply_probe(&mut self,t:u32){self.calls[self.n]=2;self.n+=1;self.last[1]=t}
        fn init_context_a(&mut self,a:u32){self.calls[self.n]=3;self.n+=1;self.last[2]=a}
        fn init_context_b(&mut self,a:u32){self.calls[self.n]=4;self.n+=1;self.last[3]=a}
        fn register(&mut self,w:u32,c:u32,k:u32,cb:u32,a4:u32,a5:u32,word:u16)->u32{self.calls[self.n]=5;self.n+=1;self.last=[w,c,k,cb,a4,a5,word as u32];self.ret}
    }
    #[test]fn init_wrapper_preserves_gate_and_arguments(){
        let mut ws=[0xAAu8;STAGE26_BT_INIT_WORKSPACE_BYTES];let mut i=I{probe_value:0x55,ret:0x1234,..I::default()};
        assert_eq!(bt_init_wrapper(&mut ws,0,0xBEEF,&mut i),0x1234);assert!(ws.iter().all(|&x|x==0));assert_eq!(&i.calls[..i.n],&[1,2,3,4,5]);
        assert_eq!(i.last,[STAGE26_BT_INIT_WORKSPACE_ADDR,STAGE26_BT_INIT_CONFIG_ADDR,23,STAGE26_BT_INIT_CALLBACK_THUMB,0,0,0xBEEF]);
        let mut ws=[1u8;STAGE26_BT_INIT_WORKSPACE_BYTES];let mut j=I::default();let _=bt_init_wrapper(&mut ws,4,7,&mut j);assert_eq!(&j.calls[..j.n],&[1,3,4,5]);
    }
    struct R{v:[(u32,u32);8],n:usize,reads:[(u32,u32);3],rn:usize}
    impl BtMmio32 for R{
        fn read32(&mut self,a:u32)->u32{let x=self.reads[self.rn];assert_eq!(x.0,a);self.rn+=1;x.1}
        fn write32(&mut self,a:u32,v:u32){self.v[self.n]=(a,v);self.n+=1}
    }
    #[test]fn post_init_register_program_exact_masks(){
        let mut r=R{v:[(0,0);8],n:0,reads:[(STAGE26_BT_MMIO_OR800_ADDR,0x20),(STAGE26_BT_MMIO_MASK_F80_ADDR,0xFFFF_FFFF),(STAGE26_BT_MMIO_MASK_F8_ADDR,0x1234_5678)],rn:0};
        assert_eq!(bt_post_init_register_program(&mut r),0);assert_eq!(r.rn,3);assert_eq!(r.n,4);
        assert_eq!(r.v[0],(STAGE26_BT_MMIO_WRITE10_ADDR,10));
        assert_eq!(r.v[1],(STAGE26_BT_MMIO_OR800_ADDR,0x820));
        assert_eq!(r.v[2],(STAGE26_BT_MMIO_MASK_F80_ADDR,(0xFFFF_FFFF&!0xF80)|0x100));
        assert_eq!(r.v[3],(STAGE26_BT_MMIO_MASK_F8_ADDR,(0x1234_5678&!0xF8)|0xE8));
    }
}

/// Stage 28: small current Bluetooth state/MMIO primitives proven by globally
/// unique relocation-normalized complete-body identity and current literal re-read.
pub const STAGE28_CURRENT_BT_INDEX_STRIDE20_ADDR:u32=0x0016_3668;
pub const STAGE28_CURRENT_BT_OBJECT_RESET_FIELDS_ADDR:u32=0x0016_5252;
pub const STAGE28_CURRENT_BT_OBJECT_BYTE18_LE_LIMIT_ADDR:u32=0x0016_53B4;
pub const STAGE28_CURRENT_BT_ENTRY_SPAN_LEN_ADDR:u32=0x0016_AA04;
pub const STAGE28_CURRENT_BT_SET_GLOBAL_60_ADDR:u32=0x0016_B7A4;
pub const STAGE28_CURRENT_BT_U32_GT_2_ADDR:u32=0x0017_0178;
pub const STAGE28_CURRENT_BT_SET_FIELD5_TO_12_ADDR:u32=0x0017_14D4;
pub const STAGE28_CURRENT_BT_POLL_SIGNED_NONNEGATIVE_100_ADDR:u32=0x0017_1DEC;
pub const STAGE28_CURRENT_BT_POLL_BIT30_SET_100_ADDR:u32=0x0017_1E04;
pub const STAGE28_CURRENT_BT_CLEAR_BIT3_ADDR:u32=0x0017_1E8C;
pub const STAGE28_CURRENT_BT_READ_BITS16_18_ADDR:u32=0x0017_1E9C;

pub const STAGE28_BT_INDEX_GLOBAL_ADDR:u32=0x0020_3160;
pub const STAGE28_BT_OBJECT_LIMIT_ADDR:u32=0x0020_3034;
pub const STAGE28_BT_ENTRY_TABLE_BASE_ADDR:u32=0x0020_D770;
pub const STAGE28_BT_GLOBAL60_ADDR:u32=0x0020_2C6D;
pub const STAGE28_BT_POLL_SIGNED_MMIO_ADDR:u32=0x0065_0318;
pub const STAGE28_BT_POLL_BIT30_MMIO_ADDR:u32=0x0065_0310;
pub const STAGE28_BT_CLEAR_BIT3_MMIO_ADDR:u32=0x0065_0314;
pub const STAGE28_BT_BITS16_18_MMIO_ADDR:u32=0x0065_031C;
pub const STAGE28_BT_ENTRY_STRIDE:usize=78;

pub const fn bt_stage28_index_stride20(index:u8)->u32{20u32*(index as u32)}
pub fn bt_stage28_set_global_60(global:&mut u8)->u32{*global=60;0}
pub const fn bt_stage28_u32_gt_2(value:u32)->bool{value>2}
pub fn bt_stage28_set_field5_to_12(field_prefix:&mut[u8;6]){field_prefix[5]=12;}

#[repr(C)]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage28ObjectPrefix{
    _pad00:[u8;18],
    pub byte18:u8,
    pub state19:u8,
    _pad20:[u8;8],
    pub word28:u32,
    _pad32:[u8;97],
    pub byte129:u8,
}
impl Default for BtStage28ObjectPrefix{
    fn default()->Self{Self{_pad00:[0;18],byte18:0,state19:0,_pad20:[0;8],word28:0,_pad32:[0;97],byte129:0}}
}
pub fn bt_stage28_object_reset_fields(object:&mut BtStage28ObjectPrefix,value:u32){object.byte129=0;object.state19=2;object.word28=value;}
pub const fn bt_stage28_object_byte18_le_limit(byte18:u8,limit:u8)->bool{byte18<=limit}
pub const fn bt_stage28_entry_span_len(entry:&[u8;STAGE28_BT_ENTRY_STRIDE])->u32{entry[11] as u32+entry[12] as u32+13}

/// Exact bounded poll: at most 100 reads; success on the first signed-nonnegative value.
pub fn bt_stage28_poll_signed_nonnegative_100<I:BtMmio32>(io:&mut I)->u32{
    let mut remaining=100u32;
    loop{
        if (io.read32(STAGE28_BT_POLL_SIGNED_MMIO_ADDR) as i32)>=0{return 1;}
        remaining-=1;if remaining==0{return 0;}
    }
}
/// Exact bounded poll: at most 100 reads; success when bit 30 becomes set.
pub fn bt_stage28_poll_bit30_set_100<I:BtMmio32>(io:&mut I)->u32{
    let mut remaining=100u32;
    loop{
        if io.read32(STAGE28_BT_POLL_BIT30_MMIO_ADDR)&0x4000_0000!=0{return 1;}
        remaining-=1;if remaining==0{return 0;}
    }
}
pub fn bt_stage28_clear_bit3<I:BtMmio32>(io:&mut I){let v=io.read32(STAGE28_BT_CLEAR_BIT3_MMIO_ADDR);io.write32(STAGE28_BT_CLEAR_BIT3_MMIO_ADDR,v&!8);}
pub fn bt_stage28_read_bits16_18<I:BtMmio32>(io:&mut I)->u32{(io.read32(STAGE28_BT_BITS16_18_MMIO_ADDR)>>16)&7}

#[cfg(test)]
mod stage28_tests{
    use super::*;use core::mem::{offset_of,size_of};
    #[test]fn pure_helpers_and_layout(){
        assert_eq!(bt_stage28_index_stride20(7),140);let mut g=0u8;assert_eq!(bt_stage28_set_global_60(&mut g),0);assert_eq!(g,60);assert!(bt_stage28_u32_gt_2(3));assert!(!bt_stage28_u32_gt_2(2));
        let mut f=[0u8;6];bt_stage28_set_field5_to_12(&mut f);assert_eq!(f[5],12);
        assert_eq!(offset_of!(BtStage28ObjectPrefix,byte18),18);assert_eq!(offset_of!(BtStage28ObjectPrefix,state19),19);assert_eq!(offset_of!(BtStage28ObjectPrefix,word28),28);assert_eq!(offset_of!(BtStage28ObjectPrefix,byte129),129);assert_eq!(size_of::<BtStage28ObjectPrefix>(),132);
        let mut o=BtStage28ObjectPrefix::default();o.byte129=9;bt_stage28_object_reset_fields(&mut o,0x11223344);assert_eq!((o.byte129,o.state19,o.word28),(0,2,0x11223344));assert!(bt_stage28_object_byte18_le_limit(4,4));assert!(!bt_stage28_object_byte18_le_limit(5,4));
        let mut e=[0u8;STAGE28_BT_ENTRY_STRIDE];e[11]=5;e[12]=7;assert_eq!(bt_stage28_entry_span_len(&e),25);
    }
    struct R{seq:[u32;100],n:usize,writes:[(u32,u32);2],wn:usize}
    impl BtMmio32 for R{fn read32(&mut self,_:u32)->u32{let v=self.seq[self.n];self.n+=1;v}fn write32(&mut self,a:u32,v:u32){self.writes[self.wn]=(a,v);self.wn+=1}}
    #[test]fn mmio_helpers_preserve_bounds_and_masks(){
        let mut r=R{seq:[0;100],n:0,writes:[(0,0);2],wn:0};r.seq[0]=0x8000_0000;r.seq[1]=7;assert_eq!(bt_stage28_poll_signed_nonnegative_100(&mut r),1);assert_eq!(r.n,2);
        let mut r=R{seq:[0x8000_0000;100],n:0,writes:[(0,0);2],wn:0};assert_eq!(bt_stage28_poll_signed_nonnegative_100(&mut r),0);assert_eq!(r.n,100);
        let mut r=R{seq:[0;100],n:0,writes:[(0,0);2],wn:0};r.seq[99]=0x4000_0000;assert_eq!(bt_stage28_poll_bit30_set_100(&mut r),1);assert_eq!(r.n,100);
        let mut r=R{seq:[0xFFFF_FFFF;100],n:0,writes:[(0,0);2],wn:0};bt_stage28_clear_bit3(&mut r);assert_eq!(r.writes[0],(STAGE28_BT_CLEAR_BIT3_MMIO_ADDR,0xFFFF_FFF7));
        let mut r=R{seq:[0x0005_0000;100],n:0,writes:[(0,0);2],wn:0};assert_eq!(bt_stage28_read_bits16_18(&mut r),5);
    }
}
