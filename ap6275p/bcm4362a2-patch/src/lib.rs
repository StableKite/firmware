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

/// Stage 29: current low-level Bluetooth programming/countdown primitives,
/// promoted only after unique relocation-normalized body identity plus current
/// literal/direct-target re-reading. Opaque ROM exits stay explicit traits.
pub const STAGE29_CURRENT_BT_PROGRAM_SOURCE_WORD_ADDR:u32=0x0017_1E1C;
pub const STAGE29_CURRENT_BT_ISSUE_PROGRAM_WORD_ADDR:u32=0x0017_1EAC;
pub const STAGE29_CURRENT_BT_PROGRAM_MASK_RETRY32_ADDR:u32=0x0017_1ED0;
pub const STAGE29_CURRENT_BT_TOGGLE_COMMAND_DISPATCH_ADDR:u32=0x0017_1FDC;
pub const STAGE29_CURRENT_BT_COUNTDOWN_DISPATCH58_ADDR:u32=0x0017_1FF8;
pub const STAGE29_BT_SOURCE_WORD_ADDR:u32=0x0022_2554;
pub const STAGE29_BT_COMMAND_BYTE_ADDR:u32=0x0022_2FD1;
pub const STAGE29_BT_TOGGLE_SOURCE_ADDR:u32=0x0020_2FD4;
pub const STAGE29_BT_MODE_ADDR:u32=0x0022_3064;
pub const STAGE29_BT_COUNTDOWN_ADDR:u32=0x0022_3065;
pub const STAGE29_BT_MMIO_STATUS_ADDR:u32=0x0065_0310;
pub const STAGE29_BT_MMIO_DATA_ADDR:u32=0x0065_0328;
pub const STAGE29_BT_MMIO_COMMAND_ADDR:u32=0x0065_0318;
pub const STAGE29_BT_MMIO_CONTROL_ADDR:u32=0x0065_0314;
pub const STAGE29_BT_MMIO_WINDOW_BASE:u32=0x0065_1000;
pub const STAGE29_BT_PROGRAM_COMMAND_MASK:u32=0x0001_FF00;
pub const STAGE29_BT_PROGRAM_COMMAND_BASE:u32=0x8500_0000;
pub const STAGE29_BT_STREAM_COMMAND:u32=0x8100_0000;
pub const STAGE29_BT_TOGGLE_TAIL_BOUNDARY:u32=0x000B_AC58;
pub const STAGE29_BT_NOTIFY58_BOUNDARY:u32=0x0002_CE90;

/// Current `0x171E1C`: if status bit 4 is clear, stream the four little-endian
/// bytes of the source word through DATA/COMMAND, polling after each byte, then
/// wait for status bit 30 and set control bit 3. Poll return values are ignored.
pub fn bt_stage29_program_source_word<I:BtMmio32>(io:&mut I,source_word:u32)->u32{
    if io.read32(STAGE29_BT_MMIO_STATUS_ADDR)&0x10!=0{return 0;}
    let mut i=0u32;
    while i<4{
        io.write32(STAGE29_BT_MMIO_DATA_ADDR,(source_word>>(i*8))&0xff);
        io.write32(STAGE29_BT_MMIO_COMMAND_ADDR,STAGE29_BT_STREAM_COMMAND);
        let _=bt_stage28_poll_signed_nonnegative_100(io);
        i+=1;
    }
    let _=bt_stage28_poll_bit30_set_100(io);
    let v=io.read32(STAGE29_BT_MMIO_CONTROL_ADDR);
    io.write32(STAGE29_BT_MMIO_CONTROL_ADDR,v|8);
    1
}

/// Current `0x171EAC`: write one value plus its encoded word index and tail
/// into the existing signed-nonnegative bounded poll.
pub fn bt_stage29_issue_program_word<I:BtMmio32>(io:&mut I,value:u32,index:u32)->u32{
    io.write32(STAGE29_BT_MMIO_DATA_ADDR,value);
    io.write32(
        STAGE29_BT_MMIO_COMMAND_ADDR,
        (index.wrapping_shl(8)&STAGE29_BT_PROGRAM_COMMAND_MASK)|STAGE29_BT_PROGRAM_COMMAND_BASE,
    );
    bt_stage28_poll_signed_nonnegative_100(io)
}

/// Current `0x171ED0`: program only bits not already present at
/// `0x651000+offset`, with at most 32 issue attempts. The original firmware
/// keeps the initially computed pending mask stable across retries.
pub fn bt_stage29_program_mask_retry32<I:BtMmio32>(io:&mut I,offset:u32,requested_mask:u32)->u32{
    let status_addr=STAGE29_BT_MMIO_WINDOW_BASE.wrapping_add(offset);
    let pending=requested_mask&!io.read32(status_addr);
    if pending==0{return 1;}
    let index=offset>>2;
    let mut remaining=32u32;
    while remaining!=0{
        let _=bt_stage29_issue_program_word(io,pending,index);
        if pending&!io.read32(status_addr)==0{return 1;}
        remaining-=1;
    }
    0
}

pub trait BtStage29ToggleTail{fn tail(&mut self,source:u8)->u32;}
/// Current `0x171FDC`: boolean-toggle the command byte, then tail-dispatch the
/// independent byte loaded from current address `0x202FD4` to opaque `0xBAC58`.
pub fn bt_stage29_toggle_command_dispatch<B:BtStage29ToggleTail>(command_byte:&mut u8,source_byte:u8,b:&mut B)->u32{
    *command_byte=if *command_byte==0{1}else{0};
    b.tail(source_byte)
}

pub trait BtStage29Notify58Boundary{fn notify(&mut self,code:&u16)->u32;}
/// Current `0x171FF8`: decrement an 8-bit counter, interpret the new byte as
/// signed, and notify opaque `0x2CE90` with a local u16 value 58 only when the
/// signed result is <=0 and mode is exactly 1 or 2. Otherwise preserve R0.
pub fn bt_stage29_countdown_dispatch58<B:BtStage29Notify58Boundary>(
    passthrough:u32,countdown:&mut u8,mode:u8,b:&mut B,
)->u32{
    let next=countdown.wrapping_sub(1);*countdown=next;
    if (next as i8)<=0 && (mode==1||mode==2){let code=58u16;b.notify(&code)}else{passthrough}
}

#[cfg(test)]
mod stage29_tests{
    use super::*;use std::vec;use std::vec::Vec;
    #[derive(Default)]struct M{reads:Vec<(u32,u32)>,ri:usize,writes:Vec<(u32,u32)>}
    impl BtMmio32 for M{
        fn read32(&mut self,a:u32)->u32{let (ea,v)=self.reads[self.ri];assert_eq!(ea,a);self.ri+=1;v}
        fn write32(&mut self,a:u32,v:u32){self.writes.push((a,v));}
    }
    #[test]fn program_source_word_exact_order_and_gate(){
        let mut blocked=M{reads:vec![(STAGE29_BT_MMIO_STATUS_ADDR,0x10)],..M::default()};
        assert_eq!(bt_stage29_program_source_word(&mut blocked,0x44332211),0);assert!(blocked.writes.is_empty());
        let mut reads=vec![(STAGE29_BT_MMIO_STATUS_ADDR,0)];for _ in 0..4{reads.push((STAGE28_BT_POLL_SIGNED_MMIO_ADDR,0));}
        reads.push((STAGE28_BT_POLL_BIT30_MMIO_ADDR,0x4000_0000));reads.push((STAGE29_BT_MMIO_CONTROL_ADDR,0x20));
        let mut m=M{reads,..M::default()};assert_eq!(bt_stage29_program_source_word(&mut m,0x44332211),1);
        assert_eq!(m.writes,vec![
            (STAGE29_BT_MMIO_DATA_ADDR,0x11),(STAGE29_BT_MMIO_COMMAND_ADDR,STAGE29_BT_STREAM_COMMAND),
            (STAGE29_BT_MMIO_DATA_ADDR,0x22),(STAGE29_BT_MMIO_COMMAND_ADDR,STAGE29_BT_STREAM_COMMAND),
            (STAGE29_BT_MMIO_DATA_ADDR,0x33),(STAGE29_BT_MMIO_COMMAND_ADDR,STAGE29_BT_STREAM_COMMAND),
            (STAGE29_BT_MMIO_DATA_ADDR,0x44),(STAGE29_BT_MMIO_COMMAND_ADDR,STAGE29_BT_STREAM_COMMAND),
            (STAGE29_BT_MMIO_CONTROL_ADDR,0x28),
        ]);
    }
    #[test]fn program_word_and_retry32(){
        let mut m=M{reads:vec![(STAGE28_BT_POLL_SIGNED_MMIO_ADDR,0)],..M::default()};
        assert_eq!(bt_stage29_issue_program_word(&mut m,0x55,3),1);
        assert_eq!(m.writes,vec![(STAGE29_BT_MMIO_DATA_ADDR,0x55),(STAGE29_BT_MMIO_COMMAND_ADDR,0x8500_0300)]);
        let status=STAGE29_BT_MMIO_WINDOW_BASE+8;
        let mut m=M{reads:vec![(status,0),(STAGE28_BT_POLL_SIGNED_MMIO_ADDR,0),(status,0x0f)],..M::default()};
        assert_eq!(bt_stage29_program_mask_retry32(&mut m,8,0x0f),1);
        let mut reads=vec![(status,0)];for _ in 0..32{reads.push((STAGE28_BT_POLL_SIGNED_MMIO_ADDR,0));reads.push((status,0));}
        let mut m=M{reads,..M::default()};assert_eq!(bt_stage29_program_mask_retry32(&mut m,8,1),0);assert_eq!(m.writes.len(),64);
    }
    struct T{seen:u8}impl BtStage29ToggleTail for T{fn tail(&mut self,s:u8)->u32{self.seen=s;0x1234}}
    struct N{seen:u16,n:u8}impl BtStage29Notify58Boundary for N{fn notify(&mut self,c:&u16)->u32{self.seen=*c;self.n+=1;0x5678}}
    #[test]fn toggle_and_signed_countdown(){
        let mut c=0u8;let mut t=T{seen:0};assert_eq!(bt_stage29_toggle_command_dispatch(&mut c,7,&mut t),0x1234);assert_eq!((c,t.seen),(1,7));
        let _=bt_stage29_toggle_command_dispatch(&mut c,8,&mut t);assert_eq!(c,0);
        let mut n=N{seen:0,n:0};let mut count=1u8;
        assert_eq!(bt_stage29_countdown_dispatch58(9,&mut count,1,&mut n),0x5678);assert_eq!((count,n.seen,n.n),(0,58,1));
        count=0;assert_eq!(bt_stage29_countdown_dispatch58(9,&mut count,2,&mut n),0x5678);assert_eq!(count,255);assert_eq!(n.n,2);
        count=2;assert_eq!(bt_stage29_countdown_dispatch58(9,&mut count,3,&mut n),9);assert_eq!(count,1);
        assert_eq!(STAGE29_CURRENT_BT_PROGRAM_SOURCE_WORD_ADDR,0x171E1C);
    }
}

/// Stage 30: current byte-program transaction lifted from the unique
/// relocation-normalized body at `0x171F04` (legacy `sub_16DE54`).  The
/// current caller at `0x16BE1C` reaches it twice for 46-byte and 1-byte
/// record updates.  Low-level MMIO programming remains composed from the
/// already verified Stage-28/29 primitives.
pub const STAGE30_CURRENT_BT_PROGRAM_BYTES_ADDR:u32=0x0017_1F04;
pub const STAGE30_CURRENT_BT_PROGRAM_BYTES_CALLER_ADDR:u32=0x0016_BE1C;
pub const STAGE30_BT_PROGRAM_OFFSET_BIAS:u32=0x0000_03C4;
pub const STAGE30_BT_PROGRAM_CALLER_RECORD_BYTES:u32=46;
pub const STAGE30_BT_PROGRAM_CALLER_SINGLE_BYTE:u32=1;

/// Abstracts only the already recovered lower-level programming primitives so
/// the transaction/chunking behavior can be tested without a physical MMIO bus.
pub trait BtStage30ProgramBackend{
    fn mode_bits(&mut self)->u32;
    fn begin_source_word(&mut self,source_word:u32);
    fn program_mask(&mut self,offset:u32,mask:u32)->u32;
    fn clear_control_bit3(&mut self);
}

pub struct BtStage30MmioBackend<'a,I:BtMmio32>{pub io:&'a mut I}
impl<I:BtMmio32> BtStage30ProgramBackend for BtStage30MmioBackend<'_,I>{
    fn mode_bits(&mut self)->u32{bt_stage28_read_bits16_18(self.io)}
    fn begin_source_word(&mut self,source_word:u32){let _=bt_stage29_program_source_word(self.io,source_word);}
    fn program_mask(&mut self,offset:u32,mask:u32)->u32{bt_stage29_program_mask_retry32(self.io,offset,mask)}
    fn clear_control_bit3(&mut self){bt_stage28_clear_bit3(self.io)}
}

fn bt_stage30_pack_word(bytes:&[u8],shift_bytes:u32)->u32{
    let mut word=0u32;let mut i=0usize;
    while i<bytes.len(){word|=(bytes[i] as u32)<<(8*i);i+=1;}
    word.wrapping_shl(8*shift_bytes)
}

/// Safe source-level model of current `0x171F04`.
///
/// `src` represents exactly the byte count supplied in R1 by the firmware
/// caller.  The current routine clears `written` first, requires mode 2,
/// begins the existing source-word programming sequence, and then programs
/// naturally aligned 32-bit masks.  A leading unaligned fragment is shifted
/// into its byte lanes; final short fragments remain zero-padded.  On the
/// first failed mask-program attempt it clears control bit 3 and returns 0.
/// Success also clears bit 3 and returns 1.  The byte counter is an 8-bit
/// firmware field and therefore wraps modulo 256.
pub fn bt_stage30_program_bytes<B:BtStage30ProgramBackend>(
    backend:&mut B,source_word:u32,offset:u32,src:&[u8],written:&mut u8,
)->u32{
    *written=0;
    if backend.mode_bits()!=2{return 0;}
    backend.begin_source_word(source_word);
    let mut address=offset.wrapping_add(STAGE30_BT_PROGRAM_OFFSET_BIAS);
    let mut pos=0usize;
    let lane=(address&3) as usize;
    if lane!=0 && pos<src.len(){
        let take=core::cmp::min(4-lane,src.len()-pos);
        let word=bt_stage30_pack_word(&src[pos..pos+take],lane as u32);
        if backend.program_mask(address&!3,word)==0{
            backend.clear_control_bit3();
            return 0;
        }
        address=address.wrapping_add(take as u32);
        pos+=take;
        *written=written.wrapping_add(take as u8);
    }
    while pos<src.len(){
        let take=core::cmp::min(4,src.len()-pos);
        let word=bt_stage30_pack_word(&src[pos..pos+take],0);
        if backend.program_mask(address&!3,word)==0{
            backend.clear_control_bit3();
            return 0;
        }
        pos+=take;
        *written=written.wrapping_add(take as u8);
        address=address.wrapping_add(4);
    }
    backend.clear_control_bit3();
    1
}

pub fn bt_stage30_program_bytes_mmio<I:BtMmio32>(
    io:&mut I,source_word:u32,offset:u32,src:&[u8],written:&mut u8,
)->u32{
    let mut backend=BtStage30MmioBackend{io};
    bt_stage30_program_bytes(&mut backend,source_word,offset,src,written)
}

#[cfg(test)]
mod stage30_tests{
    use super::*;use std::vec::Vec;
    #[derive(Default)]
    struct B{mode:u32,begins:Vec<u32>,programs:Vec<(u32,u32)>,results:Vec<u32>,ri:usize,clears:u32}
    impl BtStage30ProgramBackend for B{
        fn mode_bits(&mut self)->u32{self.mode}
        fn begin_source_word(&mut self,w:u32){self.begins.push(w)}
        fn program_mask(&mut self,o:u32,m:u32)->u32{self.programs.push((o,m));let r=self.results.get(self.ri).copied().unwrap_or(1);self.ri+=1;r}
        fn clear_control_bit3(&mut self){self.clears+=1}
    }
    #[test]fn program_bytes_preserves_alignment_count_and_failure_cleanup(){
        let mut b=B{mode:2,..B::default()};let mut written=0xAA;
        assert_eq!(bt_stage30_program_bytes(&mut b,0x4433_2211,1,&[0x11,0x22,0x33,0x44,0x55],&mut written),1);
        assert_eq!(written,5);assert_eq!(b.begins,[0x4433_2211]);
        assert_eq!(b.programs,[(0x3C4,0x3322_1100),(0x3C8,0x0000_5544)]);assert_eq!(b.clears,1);
        let mut f=B{mode:2,results:std::vec![1,0],..B::default()};written=99;
        assert_eq!(bt_stage30_program_bytes(&mut f,7,1,&[1,2,3,4,5],&mut written),0);
        assert_eq!(written,3);assert_eq!(f.programs.len(),2);assert_eq!(f.clears,1);
    }
    #[test]fn mode_gate_empty_input_and_wrapping_written_are_exact(){
        let mut b=B{mode:1,..B::default()};let mut written=9;
        assert_eq!(bt_stage30_program_bytes(&mut b,1,0,&[1,2],&mut written),0);assert_eq!(written,0);assert!(b.begins.is_empty());assert_eq!(b.clears,0);
        let mut e=B{mode:2,..B::default()};assert_eq!(bt_stage30_program_bytes(&mut e,2,0,&[],&mut written),1);assert_eq!(written,0);assert_eq!(e.begins,[2]);assert_eq!(e.clears,1);
        let src=[0u8;260];let mut w=B{mode:2,..B::default()};assert_eq!(bt_stage30_program_bytes(&mut w,3,0,&src,&mut written),1);assert_eq!(written,4);assert_eq!(w.programs.len(),65);
        assert_eq!(STAGE30_CURRENT_BT_PROGRAM_BYTES_ADDR,0x171F04);assert_eq!(STAGE30_CURRENT_BT_PROGRAM_BYTES_CALLER_ADDR,0x16BE1C);
    }
}

/// Stage 31: current 8x46-byte record command handler at `0x16BE1C`,
/// proven by the Stage-30 unique relocation-normalized caller identity plus
/// current literal/direct-target re-reading. Opaque ROM/runtime boundaries
/// remain explicit traits instead of receiving guessed vendor names.
pub const STAGE31_CURRENT_BT_RECORD_COMMAND_ADDR:u32=0x0016_BE1C;
pub const STAGE31_BT_RECORD_COUNT:usize=8;
pub const STAGE31_BT_RECORD_BYTES:usize=46;
pub const STAGE31_BT_PAYLOAD_BYTES:usize=42;
pub const STAGE31_BT_RECORD_WINDOW_OFFSET:u32=128;
pub const STAGE31_BT_RECORD_WINDOW_BYTES:usize=368;
pub const STAGE31_BT_STATUS_MMIO_ADDR:u32=0x0065_0310;
pub const STAGE31_BT_ACTIVE_MMIO_ADDR:u32=0x0064_08D8;
pub const STAGE31_BT_GUARD_WORD_ADDR:u32=0x0020_0890;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage31Request{
    pub command:u8,
    pub index:u8,
    pub payload:[u8;STAGE31_BT_PAYLOAD_BYTES],
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage31Response{
    pub status:u8,
    pub bitmap:u8,
    pub index:u8,
    pub state:u8,
    pub payload:[u8;STAGE31_BT_PAYLOAD_BYTES],
}
impl Default for BtStage31Response{
    fn default()->Self{Self{status:0,bitmap:0,index:0,state:0,payload:[0;STAGE31_BT_PAYLOAD_BYTES]}}
}

pub trait BtStage31RecordCommandBackend{
    /// Current `0x780` call shape. The same boundary is used to leave with the token.
    fn critical(&mut self,arg:u32)->u32;
    fn status_mmio(&mut self)->u32;
    /// Current `0xF620`, called only when status bit 12 is clear.
    fn prepare_record_window(&mut self);
    /// Current `0xF614`: read offset 128 / 368 bytes into the local record window.
    fn read_record_window(&mut self,offset:u32,out:&mut[u8;STAGE31_BT_RECORD_WINDOW_BYTES],shifted_status:u32);
    /// Current Stage-30 transaction at `0x171F04`.
    fn program_bytes(&mut self,offset:u32,src:&[u8],written:&mut u8)->u32;
    fn clear_active_bit0(&mut self);
}

fn bt_stage31_record_offset(index:usize)->usize{index*STAGE31_BT_RECORD_BYTES}

/// Safe source-level model of current `0x16BE1C` / legacy `sub_169548`.
///
/// Command 0 reads one record. Command 1 inserts a 46-byte record only when
/// its current state byte is zero. Command 2 programs only the state byte
/// (`0x0F`) when the current state is one. Command 3 ORs free slots into the
/// caller-provided bitmap; it intentionally does not clear pre-existing bits.
/// Unsupported commands (>3) set status 18 and return before entering the
/// opaque critical/read path, matching the firmware.
pub fn bt_stage31_record_command<B:BtStage31RecordCommandBackend>(
    req:&BtStage31Request,response:&mut BtStage31Response,b:&mut B,
){
    if req.command>3{response.status=18;return;}
    response.status=0;
    let token=b.critical(1);
    let status=b.status_mmio();
    if status&0x1000==0{b.prepare_record_window();}
    let mut records=[0u8;STAGE31_BT_RECORD_WINDOW_BYTES];
    b.read_record_window(
        STAGE31_BT_RECORD_WINDOW_OFFSET,&mut records,status.wrapping_shl(19),
    );

    let idx=req.index as usize;
    match req.command{
        0=>{
            if idx>=STAGE31_BT_RECORD_COUNT{response.status=18;}
            else{
                let o=bt_stage31_record_offset(idx);
                response.index=req.index;
                response.state=records[o+1];
                response.payload.copy_from_slice(&records[o+4..o+46]);
            }
        }
        1=>{
            if idx>=STAGE31_BT_RECORD_COUNT || records[bt_stage31_record_offset(idx)+1]!=0{
                response.status=18;
            }else{
                let mut rec=[0u8;STAGE31_BT_RECORD_BYTES];
                rec[0]=req.index;rec[1]=1;
                rec[4..].copy_from_slice(&req.payload);
                let mut written=0u8;
                let _=b.program_bytes(
                    STAGE31_BT_RECORD_WINDOW_OFFSET
                        .wrapping_add((STAGE31_BT_RECORD_BYTES*idx) as u32),
                    &rec,&mut written,
                );
                if written!=STAGE31_BT_RECORD_BYTES as u8{response.status=3;}
                response.index=req.index;
                response.state=1;
                response.payload.copy_from_slice(&req.payload);
            }
        }
        2=>{
            if idx>=STAGE31_BT_RECORD_COUNT || records[bt_stage31_record_offset(idx)+1]!=1{
                response.status=18;
            }else{
                let state=0x0Fu8;
                let mut ignored_written=0u8;
                let _=b.program_bytes(
                    STAGE31_BT_RECORD_WINDOW_OFFSET
                        .wrapping_add((STAGE31_BT_RECORD_BYTES*idx) as u32)
                        .wrapping_add(1),
                    core::slice::from_ref(&state),&mut ignored_written,
                );
            }
        }
        3=>{
            let mut i=0usize;
            while i<STAGE31_BT_RECORD_COUNT{
                if records[bt_stage31_record_offset(i)+1]==0{
                    response.bitmap|=1u8<<i;
                }
                i+=1;
            }
        }
        _=>unreachable!(),
    }
    b.clear_active_bit0();
    let _=b.critical(token);
}

#[cfg(test)]
mod stage31_tests{
    use super::*;use std::vec::Vec;
    struct B{
        status:u32,records:[u8;STAGE31_BT_RECORD_WINDOW_BYTES],critical_args:Vec<u32>,
        prepared:u32,reads:u32,programs:Vec<(u32,Vec<u8>)>,written_override:Option<u8>,clears:u32,
    }
    impl Default for B{
        fn default()->Self{Self{
            status:0,records:[0;STAGE31_BT_RECORD_WINDOW_BYTES],critical_args:Vec::new(),
            prepared:0,reads:0,programs:Vec::new(),written_override:None,clears:0,
        }}
    }
    impl BtStage31RecordCommandBackend for B{
        fn critical(&mut self,a:u32)->u32{self.critical_args.push(a);if a==1{0x55}else{0}}
        fn status_mmio(&mut self)->u32{self.status}
        fn prepare_record_window(&mut self){self.prepared+=1}
        fn read_record_window(&mut self,o:u32,out:&mut[u8;STAGE31_BT_RECORD_WINDOW_BYTES],s:u32){
            assert_eq!(o,128);assert_eq!(s,self.status.wrapping_shl(19));*out=self.records;self.reads+=1;
        }
        fn program_bytes(&mut self,o:u32,src:&[u8],written:&mut u8)->u32{
            self.programs.push((o,src.to_vec()));
            *written=self.written_override.unwrap_or(src.len() as u8);1
        }
        fn clear_active_bit0(&mut self){self.clears+=1}
    }
    fn req(c:u8,i:u8)->BtStage31Request{BtStage31Request{command:c,index:i,payload:[0xA5;42]}}
    #[test]fn read_and_invalid_command_preserve_control_flow(){
        let mut b=B::default();let o=2*46;b.records[o+1]=7;b.records[o+4..o+46].fill(0x33);
        let mut r=BtStage31Response::default();bt_stage31_record_command(&req(0,2),&mut r,&mut b);
        assert_eq!((r.status,r.index,r.state),(0,2,7));assert_eq!(r.payload,[0x33;42]);
        assert_eq!(b.critical_args,[1,0x55]);assert_eq!((b.prepared,b.reads,b.clears),(1,1,1));
        let mut b=B::default();let mut r=BtStage31Response{bitmap:0x80,..BtStage31Response::default()};
        bt_stage31_record_command(&req(4,0),&mut r,&mut b);
        assert_eq!(r.status,18);assert_eq!(r.bitmap,0x80);assert!(b.critical_args.is_empty());assert_eq!(b.reads,0);
    }
    #[test]fn insert_and_deactivate_match_record_offsets_and_status(){
        let mut b=B{status:0x1000,written_override:Some(45),..B::default()};
        let mut r=BtStage31Response::default();bt_stage31_record_command(&req(1,3),&mut r,&mut b);
        assert_eq!(r.status,3);assert_eq!((r.index,r.state),(3,1));assert_eq!(r.payload,[0xA5;42]);
        assert_eq!(b.prepared,0);assert_eq!(b.programs.len(),1);assert_eq!(b.programs[0].0,128+46*3);
        assert_eq!(b.programs[0].1[0],3);assert_eq!(b.programs[0].1[1],1);assert_eq!(&b.programs[0].1[4..],&[0xA5;42]);
        let mut b=B::default();b.records[5*46+1]=1;let mut r=BtStage31Response::default();
        bt_stage31_record_command(&req(2,5),&mut r,&mut b);
        assert_eq!(r.status,0);assert_eq!(b.programs,[(128+46*5+1,std::vec![0x0f])]);
    }
    #[test]fn free_bitmap_ors_existing_bits_and_invalid_index_cleans_up(){
        let mut b=B::default();for i in [0usize,2,7]{b.records[i*46+1]=1;}
        let mut r=BtStage31Response{bitmap:0x40,..BtStage31Response::default()};
        bt_stage31_record_command(&req(3,0),&mut r,&mut b);
        assert_eq!(r.bitmap,0x7A); // existing bit6 plus free slots 1,3,4,5,6
        let mut b=B::default();let mut r=BtStage31Response::default();
        bt_stage31_record_command(&req(0,8),&mut r,&mut b);
        assert_eq!(r.status,18);assert_eq!(b.critical_args,[1,0x55]);assert_eq!(b.clears,1);
    }
}

/// Stage 32: current record-window replay and command-class-5 adapter.
///
/// Both functions are promoted only after globally unique relocation-normalized
/// body identity plus current branch/literal re-reading. The active-record
/// callback at `0xA5D24` remains an opaque runtime boundary.
pub const STAGE32_CURRENT_BT_REPLAY_ACTIVE_RECORDS_ADDR:u32=0x0016_BF80;
pub const STAGE32_CURRENT_BT_COMMAND5_ADAPTER_ADDR:u32=0x0016_C02C;
pub const STAGE32_BT_ACTIVE_RECORD_CALLBACK_BOUNDARY:u32=0x000A_5D24;
pub const STAGE32_BT_COMMAND5_DISPATCH_TARGET:u32=STAGE25_CURRENT_BT_COMMAND_DISPATCH_ADDR;

pub trait BtStage32ReplayBackend{
    fn critical(&mut self,arg:u32)->u32;
    fn status_mmio(&mut self)->u32;
    fn prepare_record_window(&mut self);
    fn read_record_window(&mut self,offset:u32,out:&mut[u8;STAGE31_BT_RECORD_WINDOW_BYTES],shifted_status:u32);
    fn clear_active_bit0(&mut self);
    /// Current opaque `0xA5D24(0, record+4)` call shape.
    fn dispatch_active_payload(&mut self,selector:u32,payload:&[u8])->u32;
}

/// Safe source-level model of current `0x16BF80` / legacy `sub_1696AC`.
///
/// The routine snapshots the same 8x46-byte record window used by Stage 31,
/// clears the active-register bit before leaving the critical section, then
/// invokes the opaque callback for every record whose state byte (+1) is 1.
/// The callback receives selector 0 and a pointer to record byte +4 (42 bytes
/// remain in the record). The function returns the restore result when no
/// record is active, otherwise the result from the last callback.
pub fn bt_stage32_replay_active_records<B:BtStage32ReplayBackend>(b:&mut B)->u32{
    let token=b.critical(1);
    let status=b.status_mmio();
    if status&0x1000==0{b.prepare_record_window();}
    let mut records=[0u8;STAGE31_BT_RECORD_WINDOW_BYTES];
    b.read_record_window(STAGE31_BT_RECORD_WINDOW_OFFSET,&mut records,status.wrapping_shl(19));
    b.clear_active_bit0();
    let mut result=b.critical(token);
    let mut i=0usize;
    while i<STAGE31_BT_RECORD_COUNT{
        let o=i*STAGE31_BT_RECORD_BYTES;
        if records[o+1]==1{
            result=b.dispatch_active_payload(0,&records[o+4..o+STAGE31_BT_RECORD_BYTES]);
        }
        i+=1;
    }
    result
}

pub trait BtStage32Command5Dispatch{
    fn dispatch(&mut self,frame_len:u8,frame:&[u8])->u8;
}

/// Safe control-flow model of current `0x16C02C` / legacy `sub_169758`.
///
/// Request class byte +12 must equal 5. On that path byte +11 is decremented
/// with 8-bit wrap semantics and forwarded as the frame length to the already
/// recovered current command dispatcher at `0x172594`, with payload starting
/// at request byte +13. A non-class-5 request writes status 1 and preserves the
/// incoming pointer-shaped return value.
pub fn bt_stage32_command5_adapter<D:BtStage32Command5Dispatch>(
    request_passthrough:u32,parameter_len:u8,class:u8,frame:&[u8],
    response_status:&mut u8,dispatch:&mut D,
)->u32{
    if class==5{
        let result=dispatch.dispatch(parameter_len.wrapping_sub(1),frame) as u32;
        *response_status=result as u8;
        result
    }else{
        *response_status=1;
        request_passthrough
    }
}

#[cfg(test)]
mod stage32_tests{
    use super::*;use std::vec::Vec;
    struct R{
        status:u32,records:[u8;STAGE31_BT_RECORD_WINDOW_BYTES],critical_args:Vec<u32>,
        prepared:u8,read_offset:u32,shifted:u32,clears:u8,dispatches:Vec<(u32,Vec<u8>)>,next:u32,
    }
    impl Default for R{
        fn default()->Self{Self{status:0,records:[0;STAGE31_BT_RECORD_WINDOW_BYTES],critical_args:Vec::new(),prepared:0,read_offset:0,shifted:0,clears:0,dispatches:Vec::new(),next:0x9000}}
    }
    impl BtStage32ReplayBackend for R{
        fn critical(&mut self,a:u32)->u32{self.critical_args.push(a);if a==1{0x55}else{0x7777}}
        fn status_mmio(&mut self)->u32{self.status}
        fn prepare_record_window(&mut self){self.prepared+=1}
        fn read_record_window(&mut self,o:u32,out:&mut[u8;STAGE31_BT_RECORD_WINDOW_BYTES],s:u32){self.read_offset=o;self.shifted=s;*out=self.records}
        fn clear_active_bit0(&mut self){self.clears+=1}
        fn dispatch_active_payload(&mut self,sel:u32,p:&[u8])->u32{self.dispatches.push((sel,p.to_vec()));self.next+=1;self.next}
    }
    struct D{calls:Vec<(u8,Vec<u8>)>,ret:u8}
    impl BtStage32Command5Dispatch for D{fn dispatch(&mut self,n:u8,f:&[u8])->u8{self.calls.push((n,f.to_vec()));self.ret}}
    #[test]fn replay_scans_only_state_one_and_preserves_order(){
        let mut b=R::default();
        for (i,state) in [(0usize,1u8),(2,2),(5,1)]{let o=i*46;b.records[o+1]=state;b.records[o+4..o+46].fill((i+1) as u8);}
        let r=bt_stage32_replay_active_records(&mut b);
        assert_eq!(b.critical_args,[1,0x55]);assert_eq!((b.prepared,b.read_offset,b.shifted,b.clears),(1,128,0,1));
        assert_eq!(b.dispatches.len(),2);assert_eq!(b.dispatches[0],(0,std::vec![1;42]));assert_eq!(b.dispatches[1],(0,std::vec![6;42]));assert_eq!(r,0x9002);
        let mut b=R{status:0x1000,..R::default()};assert_eq!(bt_stage32_replay_active_records(&mut b),0x7777);assert_eq!(b.prepared,0);assert!(b.dispatches.is_empty());
    }
    #[test]fn command5_adapter_preserves_wrap_status_and_passthrough(){
        let frame=[9u8,8,7];let mut d=D{calls:Vec::new(),ret:3};let mut status=0xAA;
        assert_eq!(bt_stage32_command5_adapter(0x1234,4,5,&frame,&mut status,&mut d),3);assert_eq!(status,3);assert_eq!(d.calls,[(3,frame.to_vec())]);
        d.ret=7;assert_eq!(bt_stage32_command5_adapter(0x1234,0,5,&frame,&mut status,&mut d),7);assert_eq!(d.calls[1].0,255);
        let n=d.calls.len();assert_eq!(bt_stage32_command5_adapter(0xDEAD_BEEF,9,4,&frame,&mut status,&mut d),0xDEAD_BEEF);assert_eq!(status,1);assert_eq!(d.calls.len(),n);
        assert_eq!(STAGE32_CURRENT_BT_REPLAY_ACTIVE_RECORDS_ADDR,0x16BF80);assert_eq!(STAGE32_CURRENT_BT_COMMAND5_ADAPTER_ADDR,0x16C02C);assert_eq!(STAGE32_BT_COMMAND5_DISPATCH_TARGET,0x172594);
    }
}

/// Stage 33: small current control-plane helpers promoted from globally unique
/// relocation-normalized complete-body matches. Opaque ROM/runtime calls remain
/// traits; current literal addresses are recorded only as provenance anchors.
pub const STAGE33_CURRENT_BT_CLASS1_ADAPTER_ADDR:u32=0x0016_BFF4;
pub const STAGE33_CURRENT_BT_FLAG_CODE_ADDR:u32=0x0016_C228;
pub const STAGE33_CURRENT_BT_PARSE_AND_MARK_ADDR:u32=0x0016_C400;
pub const STAGE33_CURRENT_BT_INDEXED_TOGGLE_ADDR:u32=0x0016_C828;
pub const STAGE33_BT_CLASS1_MIRROR_ADDR:u32=0x0022_2708;
pub const STAGE33_BT_PARSE_CONTEXT_ADDR:u32=0x0020_CEF8;
pub const STAGE33_BT_PARSE_MARK_ADDR:u32=0x0022_2700;
pub const STAGE33_BT_INDEX_COUNT_ADDR:u32=0x0020_3160;
pub const STAGE33_BT_INDEX_METADATA_BASE_PTR_ADDR:u32=0x0020_CF10;
pub const STAGE33_BT_INDEX_FLAG_TABLE_PTR_ADDR:u32=0x0022_257C;
pub const STAGE33_BT_CLASS1_PHASE_A_BOUNDARY:u32=0x0008_9314;
pub const STAGE33_BT_CLASS1_PHASE_B_BOUNDARY:u32=0x0008_957C;
pub const STAGE33_BT_CLASS1_TAIL_BOUNDARY:u32=0x0008_9240;
pub const STAGE33_BT_PARSE_BOUNDARY:u32=0x0009_D3DC;

pub trait BtStage33Class1Boundary{
    fn phase_a(&mut self)->u32;
    fn phase_b(&mut self)->u32;
    fn tail(&mut self)->u32;
}

/// Safe control-flow model of current `0x16BFF4` / legacy `sub_169720`.
/// The response status is cleared before the class test. Class 1 runs phase A;
/// only phase-A result 1 runs phase B and then tail-dispatches. The request byte
/// at +13 is mirrored regardless of phase-A result once class 1 is accepted.
pub fn bt_stage33_class1_adapter<B:BtStage33Class1Boundary>(
    passthrough:u32,class:u8,mirror_value:u8,response_status:&mut u8,mirror:&mut u8,b:&mut B,
)->u32{
    *response_status=0;
    if class!=1{*response_status=18;return passthrough;}
    let first=b.phase_a();
    let mut result=first;
    if first==1{result=b.phase_b();}
    *mirror=mirror_value;
    if first==1{b.tail()}else{result}
}

/// Exact pure helper at current `0x16C228`: derive the firmware code from the
/// word at object offset +564. Bit 2 selects 239 vs 245; bit 6 decrements it.
pub const fn bt_stage33_flag_code(flags:u16)->u32{
    let base=if flags&4!=0{239u32}else{245u32};
    if flags&0x40!=0{base-1}else{base}
}

pub trait BtStage33ParseBoundary{fn parse(&mut self,payload:&[u8],context_addr:u32)->u8;}
/// Current `0x16C400` / legacy `sub_169A28`: feed request payload beginning at
/// +12 to opaque boundary `0x9D3DC`, store its low-byte status, and mark the
/// current global byte at `0x222700` as one.
pub fn bt_stage33_parse_and_mark<B:BtStage33ParseBoundary>(
    payload:&[u8],response_status:&mut u8,mark:&mut u8,b:&mut B,
)->u8{
    let r=b.parse(payload,STAGE33_BT_PARSE_CONTEXT_ADDR);
    *response_status=r;*mark=1;r
}

pub trait BtStage33IndexedToggleBackend{
    fn count(&self)->u8;
    fn metadata_enabled(&self,index:usize)->bool;
    fn set_flag(&mut self,index:usize,value:u8);
}
/// Current `0x16C828` / legacy `sub_169CE8`. A valid index must be below the
/// current count and metadata byte `+166` must have bit 0 set. The destination
/// is the current 20-byte-per-index flag table byte `+2`. Request value zero
/// stores one; any nonzero value stores zero. Invalid input writes status 66.
pub fn bt_stage33_indexed_toggle<B:BtStage33IndexedToggleBackend>(
    passthrough:u32,index:u16,request_value:u8,response_status:&mut u8,b:&mut B,
)->u32{
    let i=index as usize;
    if i<(b.count() as usize) && b.metadata_enabled(i){
        b.set_flag(i,if request_value==0{1}else{0});
    }else{*response_status=66;}
    passthrough
}

#[cfg(test)]
mod stage33_tests{
    use super::*;use std::vec::Vec;
    struct C{a:u32,b:u32,t:u32,calls:Vec<u8>}
    impl BtStage33Class1Boundary for C{
        fn phase_a(&mut self)->u32{self.calls.push(1);self.a}
        fn phase_b(&mut self)->u32{self.calls.push(2);self.b}
        fn tail(&mut self)->u32{self.calls.push(3);self.t}
    }
    struct P{seen:Vec<u8>,ctx:u32,ret:u8}
    impl BtStage33ParseBoundary for P{fn parse(&mut self,p:&[u8],c:u32)->u8{self.seen.extend_from_slice(p);self.ctx=c;self.ret}}
    #[derive(Default)]struct I{count:u8,enabled:[bool;4],writes:Vec<(usize,u8)>}
    impl BtStage33IndexedToggleBackend for I{
        fn count(&self)->u8{self.count}
        fn metadata_enabled(&self,i:usize)->bool{self.enabled.get(i).copied().unwrap_or(false)}
        fn set_flag(&mut self,i:usize,v:u8){self.writes.push((i,v))}
    }
    #[test]fn class1_and_flag_code_preserve_edges(){
        let mut c=C{a:0,b:7,t:9,calls:Vec::new()};let mut s=99;let mut m=0;
        assert_eq!(bt_stage33_class1_adapter(0x1234,1,0x55,&mut s,&mut m,&mut c),0);assert_eq!((s,m),(0,0x55));assert_eq!(c.calls,[1]);
        c.a=1;c.calls.clear();assert_eq!(bt_stage33_class1_adapter(0,1,2,&mut s,&mut m,&mut c),9);assert_eq!(c.calls,[1,2,3]);
        c.calls.clear();assert_eq!(bt_stage33_class1_adapter(0xDEAD,2,3,&mut s,&mut m,&mut c),0xDEAD);assert_eq!(s,18);assert!(c.calls.is_empty());
        assert_eq!(bt_stage33_flag_code(0),245);assert_eq!(bt_stage33_flag_code(4),239);assert_eq!(bt_stage33_flag_code(0x40),244);assert_eq!(bt_stage33_flag_code(0x44),238);
    }
    #[test]fn parse_mark_and_index_toggle_preserve_status_and_boolean_store(){
        let mut p=P{seen:Vec::new(),ctx:0,ret:12};let mut status=0;let mut mark=0;
        assert_eq!(bt_stage33_parse_and_mark(&[1,2,3],&mut status,&mut mark,&mut p),12);assert_eq!((status,mark,p.ctx),(12,1,STAGE33_BT_PARSE_CONTEXT_ADDR));assert_eq!(p.seen,[1,2,3]);
        let mut i=I{count:3,enabled:[true,false,true,false],writes:Vec::new()};
        assert_eq!(bt_stage33_indexed_toggle(7,0,0,&mut status,&mut i),7);assert_eq!(i.writes,[(0,1)]);
        let _=bt_stage33_indexed_toggle(8,2,9,&mut status,&mut i);assert_eq!(i.writes,[(0,1),(2,0)]);
        status=0;let _=bt_stage33_indexed_toggle(9,1,0,&mut status,&mut i);assert_eq!(status,66);
        status=0;let _=bt_stage33_indexed_toggle(9,3,0,&mut status,&mut i);assert_eq!(status,66);
        assert_eq!(STAGE33_CURRENT_BT_CLASS1_ADAPTER_ADDR,0x16BFF4);assert_eq!(STAGE33_CURRENT_BT_INDEXED_TOGGLE_ADDR,0x16C828);
    }
}

/// Stage 34: dispatch-facing wrappers around the already recovered current
/// Bluetooth slot lifecycle, plus one call-free configuration setter.
pub const STAGE34_CURRENT_BT_CLEAR_WRAPPER_ADDR:u32=0x0016_CB28;
pub const STAGE34_CURRENT_BT_INSERT_WRAPPER_ADDR:u32=0x0016_CB3E;
pub const STAGE34_CURRENT_BT_REMOVE_WRAPPER_ADDR:u32=0x0016_CB5A;
pub const STAGE34_CURRENT_BT_CONFIG_SETTER_ADDR:u32=0x0016_C978;
pub const STAGE34_BT_CLEAR_PRECHECK_BOUNDARY:u32=0x0009_B03C;
pub const STAGE34_BT_INSERT_PRECHECK_BOUNDARY:u32=0x0009_B070;
pub const STAGE34_BT_REMOVE_PRECHECK_BOUNDARY:u32=0x0009_B110;
pub const STAGE34_BT_CONFIG_WORDS_ADDR:u32=0x0022_2790;
pub const STAGE34_BT_CONFIG_BYTES_CONTEXT_ADDR:u32=0x0020_CEC4;

pub trait BtStage34LifecyclePrecheck{
    fn clear_precheck(&mut self)->u32;
    fn insert_precheck(&mut self)->u32;
    fn remove_precheck(&mut self)->u32;
}

/// Current `0x16CB28`: run opaque clear precheck and clear all slot records only
/// when the caller-owned response status byte remains zero.
pub fn bt_stage34_clear_wrapper<B:BtStage34LifecyclePrecheck>(
    records:&mut[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
    response_status:u8,b:&mut B,
)->u32{
    let r=b.clear_precheck();
    if response_status==0{bt_clear_slot_table(records);}
    r
}

/// Current `0x16CB3E`: run opaque insert precheck; when status remains zero,
/// insert request tag + six-byte payload and copy the firmware insert result to
/// response status.
pub fn bt_stage34_insert_wrapper<B:BtStage34LifecyclePrecheck>(
    tag:u8,payload:&[u8;STAGE23_BT_SLOT_PAYLOAD_BYTES],records:&mut[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
    response_status:&mut u8,b:&mut B,
)->u32{
    let r=b.insert_precheck();
    if *response_status==0{
        let s=bt_insert_slot_if_absent(records,tag,payload);
        *response_status=s;
        s as u32
    }else{r}
}

/// Current `0x16CB5A`: run opaque remove precheck and, only on zero response
/// status, tail into the already recovered current remove operation. Unlike the
/// insert wrapper, the remove result is not copied into response status here.
pub fn bt_stage34_remove_wrapper<B:BtStage34LifecyclePrecheck>(
    tag:u8,payload:&[u8;STAGE23_BT_SLOT_PAYLOAD_BYTES],records:&mut[[u8;STAGE22_BT_SLOT_RECORD_BYTES];STAGE22_BT_SLOT_COUNT],
    response_status:u8,b:&mut B,
)->u32{
    let r=b.remove_precheck();
    if response_status==0{bt_remove_slot(records,tag,payload) as u32}else{r}
}

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage34ConfigState{pub word0:u16,pub word1:u16,pub byte31:u8,pub byte32:u8}
/// Exact call-free model of current `0x16C978` / legacy `sub_169E38`.
/// Request bytes +13..+16 become two little-endian words; +12 and +17 become
/// two independent bytes in the second current context object.
pub fn bt_stage34_set_config(state:&mut BtStage34ConfigState,request:&[u8;18]){
    state.word0=u16::from_le_bytes([request[13],request[14]]);
    state.word1=u16::from_le_bytes([request[15],request[16]]);
    state.byte31=request[12];
    state.byte32=request[17];
}

#[cfg(test)]
mod stage34_tests{
    use super::*;use std::vec::Vec;
    struct B{ret:[u32;3],calls:Vec<u8>}
    impl BtStage34LifecyclePrecheck for B{
        fn clear_precheck(&mut self)->u32{self.calls.push(0);self.ret[0]}
        fn insert_precheck(&mut self)->u32{self.calls.push(1);self.ret[1]}
        fn remove_precheck(&mut self)->u32{self.calls.push(2);self.ret[2]}
    }
    #[test]fn lifecycle_wrappers_preserve_status_gates_and_return_shapes(){
        let mut b=B{ret:[10,11,12],calls:Vec::new()};let mut records=[[1u8;7];8];
        assert_eq!(bt_stage34_clear_wrapper(&mut records,1,&mut b),10);assert_eq!(records,[[1;7];8]);
        assert_eq!(bt_stage34_clear_wrapper(&mut records,0,&mut b),10);assert_eq!(records,[[0;7];8]);
        let p=[1,2,3,4,5,6];let mut s=0u8;assert_eq!(bt_stage34_insert_wrapper(7,&p,&mut records,&mut s,&mut b),0);assert_eq!(s,0);assert_eq!(records[0],[7,1,2,3,4,5,6]);
        s=18;assert_eq!(bt_stage34_insert_wrapper(8,&p,&mut records,&mut s,&mut b),11);assert_eq!(s,18);
        assert_eq!(bt_stage34_remove_wrapper(7,&p,&mut records,0,&mut b),1);assert_eq!(bt_stage34_remove_wrapper(7,&p,&mut records,9,&mut b),12);
        assert_eq!(b.calls,[0,0,1,1,2,2]);
    }
    #[test]fn config_setter_preserves_exact_byte_layout(){
        let mut req=[0u8;18];req[12]=0xAA;req[13]=0x34;req[14]=0x12;req[15]=0x78;req[16]=0x56;req[17]=0xBB;
        let mut s=BtStage34ConfigState::default();bt_stage34_set_config(&mut s,&req);
        assert_eq!(s,BtStage34ConfigState{word0:0x1234,word1:0x5678,byte31:0xAA,byte32:0xBB});
        assert_eq!(STAGE34_CURRENT_BT_CLEAR_WRAPPER_ADDR,0x16CB28);assert_eq!(STAGE34_CURRENT_BT_CONFIG_SETTER_ADDR,0x16C978);
    }
}

/// Stage 35: current local eligibility/mask helpers around `0x16D3B8..0x16D4D4`.
///
/// All four functions are globally unique relocation-normalized matches. The
/// last two are byte-identical legacy/current bodies. Unresolved external
/// calls stay explicit traits rather than receiving guessed vendor names.
pub const STAGE35_CURRENT_BT_MASK_LOOKUP_ADDR:u32=0x0016_D3B8;
pub const STAGE35_CURRENT_BT_ELIGIBILITY_GATE_ADDR:u32=0x0016_D3D8;
pub const STAGE35_CURRENT_BT_CLEAR_MASK_BIT_ADDR:u32=0x0016_D450;
pub const STAGE35_CURRENT_BT_MATCH_RECORD_ADDR:u32=0x0016_D490;
pub const STAGE35_BT_MASK_LOOKUP_TABLE_ADDR:u32=0x0022_1EBC;
pub const STAGE35_BT_ENABLED_MASK_ADDR:u32=0x0022_1EC4;
pub const STAGE35_BT_INDEX_MASK_TABLE_ADDR:u32=0x0022_1EC6;
pub const STAGE35_BT_MATCH_TABLE_ADDR:u32=0x0020_9D68;
pub const STAGE35_BT_GLOBAL_FEATURE_CONTEXT_ADDR:u32=0x0020_8338;
pub const STAGE35_BT_CONTEXT_PROBE_ADDR:u32=0x0020_91FC;
pub const STAGE35_BT_RESET_CONTEXT_ADDR:u32=0x0020_9454;
pub const STAGE35_BT_SPECIAL_CONTEXT_ADDR:u32=0x0020_8194;
pub const STAGE35_BT_MAP_INDEX_BOUNDARY:u32=0x0003_2DD4;
pub const STAGE35_BT_CONTEXT_BUSY_BOUNDARY:u32=0x0002_1F42;
pub const STAGE35_BT_ZERO_PROBE_BOUNDARY:u32=0x0002_A428;
pub const STAGE35_BT_POST_ZERO_BOUNDARY:u32=0x0002_A2B8;

pub trait BtStage35MaskLookupBackend{
    fn map_index(&mut self,selector:u32)->u32;
    fn mapped_value(&mut self,index:u32)->u8;
}

/// Current `0x16D3B8`: call the opaque index mapper unconditionally, then
/// return the mapped byte only when object word `+36` contains mask `0x110`.
/// Otherwise return zero. The unconditional mapper call is intentional.
pub fn bt_stage35_mask_lookup<B:BtStage35MaskLookupBackend>(
    mask_word36:u16,selector:u32,b:&mut B,
)->u32{
    let index=b.map_index(selector);
    if mask_word36&0x0110!=0{b.mapped_value(index) as u32}else{0}
}

pub trait BtStage35EligibilityBackend:BtStage35MaskLookupBackend{
    fn context_busy(&mut self,context_addr:u32)->u32;
    fn zero_probe(&mut self)->u32;
    fn post_zero(&mut self);
}

/// Safe control-flow model of current `0x16D3D8` / legacy `sub_16A65C`.
///
/// The four external runtime contracts remain opaque. This function preserves
/// their ordering, all early exits, the 24-bit object-word check, the current
/// reset-latch zero store, and the final three-way special-object predicate.
pub fn bt_stage35_eligibility_gate<B:BtStage35EligibilityBackend>(
    global_feature:u8,object_word12:u32,mask_word36:u16,selector:u32,
    reset_gate:u8,reset_latch:&mut u32,special_enabled:u8,
    object_identity:u32,peer_code:u16,special_object_a:u32,special_object_b:u32,
    b:&mut B,
)->u32{
    if global_feature&0x80!=0{return 1;}
    if object_word12&0x00FF_FFFF!=0{return 1;}
    if bt_stage35_mask_lookup(mask_word36,selector,b)!=0{return 1;}
    if b.context_busy(STAGE35_BT_CONTEXT_PROBE_ADDR)!=0{return 1;}
    let probe=b.zero_probe();
    if probe!=0{return 1;}
    if reset_gate!=0{return 1;}
    *reset_latch=0;
    b.post_zero();
    if special_enabled!=0 &&
        (peer_code==0x080B || object_identity==special_object_a || object_identity==special_object_b)
    {return 1;}
    probe
}

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage35BitObject{
    pub word12:u16,
    pub index14:u8,
    pub mask36:u16,
}
pub trait BtStage35IndexMaskTable{fn index_mask(&mut self,index:u8)->u16;}

fn bt_stage35_arm_register_lsl_one(shift:u32)->u32{
    let amount=shift&0xFF;
    if amount<32{1u32<<amount}else{0}
}
fn bt_stage35_arm_register_shift_positive_u16(value:u16,shift:u32)->u32{
    let amount=shift&0xFF;
    if amount<32{(value as u32)>>amount}else{0}
}

/// Exact call-free model of current `0x16D450` / legacy `sub_16A6D4`.
///
/// It computes `1 << bit_index` with Thumb register-shift semantics, requires
/// that low-16 bit in the current enabled mask, optionally preserves the bit
/// when either the shifted object word has bit zero set or the indexed mask
/// equals it, and otherwise clears the bit in object word `+36`.
pub fn bt_stage35_clear_mask_bit<T:BtStage35IndexMaskTable>(
    passthrough:u32,object:&mut BtStage35BitObject,mode_nonzero:bool,
    guard_shift:u32,bit_index:u32,enabled_mask:u16,table:&mut T,
)->u32{
    let full=bt_stage35_arm_register_lsl_one(bit_index);
    let bit=full as u16;
    if bit&enabled_mask!=0{
        if guard_shift!=0{
            if mode_nonzero{
                if bt_stage35_arm_register_shift_positive_u16(object.word12,guard_shift)&1!=0{
                    return passthrough;
                }
            }else if table.index_mask(object.index14)==bit{
                return passthrough;
            }
        }
        object.mask36&=!bit;
    }
    passthrough
}

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage35MatchRecord{
    pub byte209:u8,
    pub byte215:u8,
    pub byte229:u8,
}

/// Exact call-free model of current `0x16D490` / legacy `sub_16A714`.
/// The firmware scans exactly three 400-byte-stride records and succeeds when
/// byte +209 has bit `0x10`, wrapped `(byte+229 + 29) & 31` is at most three,
/// and byte +215 equals the full 32-bit key value.
pub fn bt_stage35_any_matching_record(key:u32,records:&[BtStage35MatchRecord;3])->u32{
    let mut i=0usize;
    while i<3{
        let r=records[i];
        if r.byte209&0x10!=0 && (r.byte229.wrapping_add(29)&0x1F)<=3 && r.byte215 as u32==key{
            return 1;
        }
        i+=1;
    }
    0
}

#[cfg(test)]
mod stage35_tests{
    use super::*;use std::vec::Vec;
    struct B{map:u32,mapped:u8,busy:u32,probe:u32,calls:Vec<u8>}
    impl BtStage35MaskLookupBackend for B{
        fn map_index(&mut self,_:u32)->u32{self.calls.push(1);self.map}
        fn mapped_value(&mut self,_:u32)->u8{self.calls.push(2);self.mapped}
    }
    impl BtStage35EligibilityBackend for B{
        fn context_busy(&mut self,a:u32)->u32{assert_eq!(a,STAGE35_BT_CONTEXT_PROBE_ADDR);self.calls.push(3);self.busy}
        fn zero_probe(&mut self)->u32{self.calls.push(4);self.probe}
        fn post_zero(&mut self){self.calls.push(5)}
    }
    struct T{values:[u16;4],calls:Vec<u8>}
    impl BtStage35IndexMaskTable for T{fn index_mask(&mut self,i:u8)->u16{self.calls.push(i);self.values[i as usize]}}
    #[test]fn lookup_and_eligibility_preserve_order_and_early_exits(){
        let mut b=B{map:2,mapped:7,busy:0,probe:0,calls:Vec::new()};
        assert_eq!(bt_stage35_mask_lookup(0,9,&mut b),0);assert_eq!(b.calls,[1]);
        b.calls.clear();assert_eq!(bt_stage35_mask_lookup(0x10,9,&mut b),7);assert_eq!(b.calls,[1,2]);
        let mut latch=0xAAAAu32;b.mapped=0;b.calls.clear();
        assert_eq!(bt_stage35_eligibility_gate(0,0,0,3,0,&mut latch,0,10,7,11,12,&mut b),0);
        assert_eq!(latch,0);assert_eq!(b.calls,[1,3,4,5]);
        latch=9;b.calls.clear();assert_eq!(bt_stage35_eligibility_gate(0x80,0,0,0,0,&mut latch,0,0,0,0,0,&mut b),1);assert_eq!(latch,9);assert!(b.calls.is_empty());
        b.calls.clear();assert_eq!(bt_stage35_eligibility_gate(0,1,0,0,0,&mut latch,0,0,0,0,0,&mut b),1);assert!(b.calls.is_empty());
        b.calls.clear();b.mapped=1;assert_eq!(bt_stage35_eligibility_gate(0,0,0x100,0,0,&mut latch,0,0,0,0,0,&mut b),1);assert_eq!(b.calls,[1,2]);
        b.mapped=0;b.calls.clear();assert_eq!(bt_stage35_eligibility_gate(0,0,0,0,0,&mut latch,1,11,0x080B,1,2,&mut b),1);assert_eq!(b.calls,[1,3,4,5]);
    }
    #[test]fn bit_clear_and_three_record_match_preserve_machine_edges(){
        let mut o=BtStage35BitObject{word12:1,index14:2,mask36:0xFFFF};let mut t=T{values:[0,0,2,0],calls:Vec::new()};
        assert_eq!(bt_stage35_clear_mask_bit(0x55,&mut o,true,256,1,2,&mut t),0x55);assert_eq!(o.mask36,0xFFFF);assert!(t.calls.is_empty());
        // shift 256 is nonzero to the branch but has ARM register-shift amount zero.
        o.word12=0;o.mask36=0xFFFF;bt_stage35_clear_mask_bit(0,&mut o,false,1,1,2,&mut t);assert_eq!(o.mask36,0xFFFF);assert_eq!(t.calls,[2]);
        t.values[2]=0;o.mask36=0xFFFF;bt_stage35_clear_mask_bit(0,&mut o,false,1,1,2,&mut t);assert_eq!(o.mask36,0xFFFD);
        o.mask36=0xFFFF;bt_stage35_clear_mask_bit(0,&mut o,false,0,40,0xFFFF,&mut t);assert_eq!(o.mask36,0xFFFF);
        let mut rs=[BtStage35MatchRecord::default();3];rs[1]=BtStage35MatchRecord{byte209:0x10,byte215:7,byte229:3};
        assert_eq!(bt_stage35_any_matching_record(7,&rs),1);assert_eq!(bt_stage35_any_matching_record(0x107,&rs),0);
        rs[1].byte229=7;assert_eq!(bt_stage35_any_matching_record(7,&rs),0);
        assert_eq!(STAGE35_CURRENT_BT_MASK_LOOKUP_ADDR,0x16D3B8);assert_eq!(STAGE35_CURRENT_BT_MATCH_RECORD_ADDR,0x16D490);
    }
}

/// Stage 36: current 440-byte window transaction at `0x16C240`.
///
/// The complete current body is a globally unique relocation-normalized match
/// of legacy `sub_169868`. Runtime services that are not independently
/// identified remain explicit traits; the known flag-code helper is composed
/// from Stage 33.
pub const STAGE36_CURRENT_BT_WINDOW_TRANSACTION_ADDR:u32=0x0016_C240;
pub const STAGE36_BT_WINDOW_BASE_PTR_ADDR:u32=0x0020_CE98;
pub const STAGE36_BT_GROUP_BASE_PTR_ADDR:u32=0x0020_BE7C;
pub const STAGE36_BT_WINDOW_STRIDE:u32=1028;
pub const STAGE36_BT_GROUP_STRIDE:u32=676;
pub const STAGE36_BT_READY_BOUNDARY:u32=0x0008_E12C;
pub const STAGE36_BT_CONTEXT_BOUNDARY:u32=0x0008_86E0;
pub const STAGE36_BT_VALIDATE_BOUNDARY:u32=0x0009_D7C4;
pub const STAGE36_BT_CRITICAL_BOUNDARY:u32=0x0000_0780;
pub const STAGE36_BT_TRANSFORM_LOW12_BOUNDARY:u32=0x0008_F160;
pub const STAGE36_BT_NOTIFY_GROUP_BOUNDARY:u32=0x0007_9AAE;
pub const STAGE36_BT_MEMCPY_BOUNDARY:u32=ROM_MEMCPY_ADDR;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage36Context{
    pub word552:u16,
    pub byte19:u8,
    pub byte554:u8,
    pub byte555:u8,
    pub byte556:u8,
    pub flags564:u16,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage36Request<'a>{
    /// Request bytes +9..+11, visible to the still-opaque validator.
    pub prefix9_11:[u8;3],
    pub selector:u8,
    pub operation:u8,
    pub byte14:u8,
    pub length:u8,
    /// Bytes beginning at firmware request offset +16. Stage 36 itself never
    /// indexes this slice; copies are delegated to the backend with exact
    /// firmware offset/length arguments.
    pub payload:&'a[u8],
}

pub trait BtStage36WindowBackend{
    /// Current `0x8E12C` readiness-like boundary.
    fn ready(&mut self)->u32;
    /// Current `0x886E0`; `false` represents its null result.
    fn context_present(&mut self)->bool;
    /// Current `0x9D7C4(request+9, context, window, special)` call shape.
    fn validate(&mut self,request:&BtStage36Request<'_>,context:&BtStage36Context,window_index:u8,special:u8)->u32;
    fn read_header16(&mut self,window_index:u8)->u16;
    fn write_header16(&mut self,window_index:u8,value:u16);
    fn read_header32(&mut self,window_index:u8)->u32;
    fn write_header32(&mut self,window_index:u8,value:u32);
    fn read_header_byte2(&mut self,window_index:u8)->u8;
    fn write_header_byte2(&mut self,window_index:u8,value:u8);
    /// Current critical-state exchange at `0x780`.
    fn critical(&mut self,arg:u32)->u32;
    /// Current `0x8F160`, called with the low 12 bits of context word +552.
    fn transform_low12(&mut self,value:u16)->u32;
    /// Current `0x79AAE` on the 676-byte-stride group selected by window index.
    fn notify_group(&mut self,window_index:u8);
    /// Known memcpy-like `0x3DB4`, with the request source beginning at +16.
    fn copy_request_payload(&mut self,request:&BtStage36Request<'_>,window_index:u8,offset:u16,length:u8);
}

fn bt_stage36_replace_low12(old:u16,value:u32)->u16{
    (old&0xF000)|((value as u16)&0x0FFF)
}
fn bt_stage36_replace_low11_u16(old:u16,value:u32)->u16{
    (old&!0x07FF)|((value as u16)&0x07FF)
}
fn bt_stage36_replace_bits11_21(old:u32,value:u32)->u32{
    (old&!(0x07FFu32<<11))|((value&0x07FF)<<11)
}

/// Safe source-level model of current `0x16C240` / legacy `sub_169868`.
///
/// The routine validates selector/window state, preserves the firmware's
/// status/error return shapes, optionally updates a 1028-byte-stride window,
/// and restores the critical-state token on every entered-critical path.
/// Successful noncritical `special != 0` exits intentionally leave
/// `response_status` untouched.
pub fn bt_stage36_window_transaction<B:BtStage36WindowBackend>(
    request:&BtStage36Request<'_>,context:&mut BtStage36Context,
    response_status:&mut u8,b:&mut B,
)->u32{
    if b.ready()==0{
        *response_status=1;
        return 0;
    }

    let mut result=request.selector as u32;
    if result>0xEF{
        *response_status=18;
        return result;
    }
    if !b.context_present(){
        *response_status=66;
        return 0;
    }

    let window_index=(context.byte555>>3)&0x0F;
    let special=(context.byte556>>2)&1;
    result=b.validate(request,context,window_index,special);
    if result!=0{
        *response_status=result as u8;
        return result;
    }

    let flags=context.flags564;
    if flags&0x10==0{
        result=bt_stage33_flag_code(flags);
        if context.byte554&0x30==0x10{
            let opmask=request.operation&0xFD;
            if opmask==1{
                if result<(request.length as u32){
                    *response_status=18;
                    return result;
                }
            }else if opmask==0{
                let used=(b.read_header16(window_index)&0x07FF) as u32;
                if (request.length as u32)+used>result{
                    *response_status=18;
                    return result;
                }
            }
        }
    }

    if ((flags&0x12)==2 || (flags&0x14)==0x14)
        && (request.operation!=3 || request.length!=0)
    {
        *response_status=18;
        return result;
    }

    if special!=0{return result;}

    let token=b.critical(1);
    if request.operation==4{
        let transformed=b.transform_low12(context.word552&0x0FFF);
        context.word552=bt_stage36_replace_low12(context.word552,transformed);
        return b.critical(token);
    }

    let opmask=request.operation&0xFD;
    if opmask==1{
        // Firmware first writes the low halfword, then performs a full-word
        // BFI of bits 11..21. Preserve that write sequence explicitly.
        let h16=b.read_header16(window_index);
        b.write_header16(window_index,bt_stage36_replace_low11_u16(h16,request.length as u32));
        let h32=b.read_header32(window_index);
        b.write_header32(window_index,bt_stage36_replace_bits11_21(h32,special as u32));

        if context.byte19!=0{b.notify_group(window_index);}
        if context.byte554&0x30!=0x20{
            let transformed=b.transform_low12(context.word552&0x0FFF);
            context.word552=bt_stage36_replace_low12(context.word552,transformed);
        }
        if request.length!=0{
            b.copy_request_payload(request,window_index,0,request.length);
        }

        let byte2=b.read_header_byte2(window_index)|0x40;
        b.write_header_byte2(window_index,byte2);
        let byte2=b.read_header_byte2(window_index);
        b.write_header_byte2(
            window_index,
            if request.operation==3{byte2|0x80}else{byte2&0x7F},
        );
    }else{
        if request.length!=0{
            // Destination offset is captured before the copy. The firmware
            // re-reads the header after the copy before adding the length.
            let offset=b.read_header16(window_index)&0x07FF;
            b.copy_request_payload(request,window_index,offset,request.length);
            let h16=b.read_header16(window_index);
            let next=((h16&0x07FF) as u32).wrapping_add(request.length as u32);
            b.write_header16(window_index,bt_stage36_replace_low11_u16(h16,next));
        }
        if request.operation==2{
            let byte2=b.read_header_byte2(window_index);
            b.write_header_byte2(window_index,byte2|0x80);
        }
    }

    b.critical(token)
}

#[cfg(test)]
mod stage36_tests{
    use super::*;use std::vec::Vec;
    #[derive(Clone,Debug,PartialEq,Eq)]enum E{Ready,Present,Validate(u8,u8),R16(u8),W16(u8,u16),R32(u8),W32(u8,u32),RB2(u8),WB2(u8,u8),Crit(u32),Xform(u16),Notify(u8),Copy(u8,u16,u8)}
    struct B{ready:u32,present:bool,validate:u32,h16:u16,h32:u32,b2:u8,token:u32,xform:u32,events:Vec<E>}
    impl Default for B{fn default()->Self{Self{ready:1,present:true,validate:0,h16:0,h32:0,b2:0,token:0x55,xform:0x456,events:Vec::new()}}}
    impl BtStage36WindowBackend for B{
        fn ready(&mut self)->u32{self.events.push(E::Ready);self.ready}
        fn context_present(&mut self)->bool{self.events.push(E::Present);self.present}
        fn validate(&mut self,r:&BtStage36Request<'_>,_:&BtStage36Context,i:u8,s:u8)->u32{self.events.push(E::Validate(i,s));assert_eq!(r.prefix9_11,[9,10,11]);self.validate}
        fn read_header16(&mut self,i:u8)->u16{self.events.push(E::R16(i));self.h16}
        fn write_header16(&mut self,i:u8,v:u16){self.events.push(E::W16(i,v));self.h16=v;self.h32=(self.h32&0xFFFF_0000)|v as u32}
        fn read_header32(&mut self,i:u8)->u32{self.events.push(E::R32(i));self.h32}
        fn write_header32(&mut self,i:u8,v:u32){self.events.push(E::W32(i,v));self.h32=v;self.h16=v as u16;self.b2=(v>>16) as u8}
        fn read_header_byte2(&mut self,i:u8)->u8{self.events.push(E::RB2(i));self.b2}
        fn write_header_byte2(&mut self,i:u8,v:u8){self.events.push(E::WB2(i,v));self.b2=v;self.h32=(self.h32&!(0xFF<<16))|((v as u32)<<16)}
        fn critical(&mut self,a:u32)->u32{self.events.push(E::Crit(a));if a==1{self.token}else{0x7777}}
        fn transform_low12(&mut self,v:u16)->u32{self.events.push(E::Xform(v));self.xform}
        fn notify_group(&mut self,i:u8){self.events.push(E::Notify(i))}
        fn copy_request_payload(&mut self,_:&BtStage36Request<'_>,i:u8,o:u16,n:u8){self.events.push(E::Copy(i,o,n))}
    }
    fn req(op:u8,len:u8)->BtStage36Request<'static>{BtStage36Request{prefix9_11:[9,10,11],selector:7,operation:op,byte14:0,length:len,payload:&[]}}
    #[test]fn early_gates_preserve_status_and_result_shapes(){
        let mut c=BtStage36Context::default();let mut s=0xAA;let mut b=B{ready:0,..B::default()};
        assert_eq!(bt_stage36_window_transaction(&req(0,0),&mut c,&mut s,&mut b),0);assert_eq!(s,1);assert_eq!(b.events,[E::Ready]);
        let r=BtStage36Request{selector:240,..req(0,0)};let mut b=B::default();s=0;
        assert_eq!(bt_stage36_window_transaction(&r,&mut c,&mut s,&mut b),240);assert_eq!(s,18);assert_eq!(b.events,[E::Ready]);
        let mut b=B{present:false,..B::default()};s=9;assert_eq!(bt_stage36_window_transaction(&req(0,0),&mut c,&mut s,&mut b),0);assert_eq!(s,66);
        let mut b=B{validate:0x123,..B::default()};s=9;assert_eq!(bt_stage36_window_transaction(&req(0,0),&mut c,&mut s,&mut b),0x123);assert_eq!(s,0x23);
    }
    #[test]fn replace_append_special_and_op4_preserve_update_order(){
        let mut c=BtStage36Context{word552:0xA123,byte19:1,byte554:0,byte555:0x18,byte556:0,flags564:0x10};
        let mut b=B{h16:0xF123,h32:0xAA55_F123,b2:0x55,..B::default()};let mut s=0xCC;
        assert_eq!(bt_stage36_window_transaction(&req(1,4),&mut c,&mut s,&mut b),0x7777);assert_eq!(s,0xCC);assert_eq!(c.word552,0xA456);
        assert!(b.events.contains(&E::Notify(3)));assert!(b.events.contains(&E::Copy(3,0,4)));assert_eq!(b.b2&0xC0,0x40);
        let mut c=BtStage36Context{byte555:0x08,flags564:0x10,..BtStage36Context::default()};let mut b=B{h16:5,h32:5,b2:0,..B::default()};
        assert_eq!(bt_stage36_window_transaction(&req(2,3),&mut c,&mut s,&mut b),0x7777);assert!(b.events.contains(&E::Copy(1,5,3)));assert_eq!(b.h16&0x7ff,8);assert_eq!(b.b2&0x80,0x80);
        let mut c=BtStage36Context{byte556:4,flags564:0,..BtStage36Context::default()};let mut b=B::default();s=0x5A;
        assert_eq!(bt_stage36_window_transaction(&req(0,0),&mut c,&mut s,&mut b),245);assert_eq!(s,0x5A);assert!(!b.events.iter().any(|e|matches!(e,E::Crit(_))));
        let mut c=BtStage36Context{word552:0xB123,flags564:0x10,..BtStage36Context::default()};let mut b=B{xform:0xABC,..B::default()};
        assert_eq!(bt_stage36_window_transaction(&req(4,0),&mut c,&mut s,&mut b),0x7777);assert_eq!(c.word552,0xBABC);assert!(b.events.contains(&E::Xform(0x123)));
    }
    #[test]fn capacity_and_restricted_rejections_return_current_result(){
        let mut c=BtStage36Context{byte554:0x10,flags564:0,..BtStage36Context::default()};let mut b=B::default();let mut s=0;
        assert_eq!(bt_stage36_window_transaction(&req(1,246),&mut c,&mut s,&mut b),245);assert_eq!(s,18);
        b=B{h16:240,..B::default()};s=0;assert_eq!(bt_stage36_window_transaction(&req(0,6),&mut c,&mut s,&mut b),245);assert_eq!(s,18);
        c=BtStage36Context{flags564:2,..BtStage36Context::default()};b=B::default();s=0;assert_eq!(bt_stage36_window_transaction(&req(1,0),&mut c,&mut s,&mut b),245);assert_eq!(s,18);
        assert_eq!(STAGE36_CURRENT_BT_WINDOW_TRANSACTION_ADDR,0x16C240);assert_eq!(STAGE36_BT_WINDOW_STRIDE,1028);assert_eq!(STAGE36_BT_GROUP_STRIDE,676);
    }
}

/// Stage 37: current request-to-config update cluster at
/// `0x16C870`, `0x16C8E4`, and `0x16C942`.
///
/// Each complete current body is a globally unique relocation-normalized match
/// of its legacy counterpart. Lookup/config/commit services remain opaque traits.
pub const STAGE37_CURRENT_BT_INDEXED_CONFIG_ADDR:u32=0x0016_C870;
pub const STAGE37_CURRENT_BT_SELECTOR_CONFIG_ADDR:u32=0x0016_C8E4;
pub const STAGE37_CURRENT_BT_FIELD_UPDATE_ADDR:u32=0x0016_C942;
pub const STAGE37_BT_INDEX_COUNT_ADDR:u32=STAGE33_BT_INDEX_COUNT_ADDR;
pub const STAGE37_BT_INDEX_METADATA_BASE_PTR_ADDR:u32=STAGE33_BT_INDEX_METADATA_BASE_PTR_ADDR;
pub const STAGE37_BT_LOOKUP_BOUNDARY:u32=0x0008_D34C;
pub const STAGE37_BT_CONTEXT_BOUNDARY:u32=0x0008_86E0;
pub const STAGE37_BT_CONFIG_BOUNDARY:u32=0x0016_3724;
pub const STAGE37_BT_COMMIT_BOUNDARY:u32=0x0016_1268;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage37Request{
    pub key:u16,
    pub byte14:u8,
    pub byte15:u8,
    pub byte16:u8,
    pub byte17:u8,
    pub byte18:u8,
    pub byte19:u8,
}
impl BtStage37Request{
    pub const fn word14(self)->u16{(self.byte14 as u16)|((self.byte15 as u16)<<8)}
    pub const fn word15(self)->u16{(self.byte15 as u16)|((self.byte16 as u16)<<8)}
    pub const fn word16(self)->u16{(self.byte16 as u16)|((self.byte17 as u16)<<8)}
    pub const fn word17(self)->u16{(self.byte17 as u16)|((self.byte18 as u16)<<8)}
}

pub trait BtStage37ConfigBackend{
    fn index_count(&self)->u8;
    fn metadata_enabled(&self,index:usize)->bool;
    /// Current `0x886E0`; zero means no current context.
    fn current_context(&mut self)->u32;
    fn context_byte592(&mut self,context:u32)->u8;
    /// Current `0x8D34C` object lookup; zero means not found.
    fn lookup_object(&mut self,key:u16)->u32;
    fn object_byte223(&mut self,object:u32)->u8;
    /// Current `0x163724` config-object boundary.
    fn config_object(&mut self,object:u32)->u32;
    fn write_config_u16(&mut self,config:u32,offset:u8,value:u16);
    fn write_config_u8(&mut self,config:u32,offset:u8,value:u8);
    /// Current `0x161268` tail/commit boundary.
    fn commit_object(&mut self,object:u32)->u32;
}

/// Current `0x16C870` / legacy `sub_169D30`.
///
/// Request word +16 is first checked against the current global count and the
/// 264-byte-stride metadata bit at +166. The object lookup then requires byte
/// +223 bit 1. Success writes request words +14/+16 into config offsets +2/+4,
/// sets config byte +11 to one, and tail-dispatches the object commit boundary.
pub fn bt_stage37_indexed_config<B:BtStage37ConfigBackend>(
    request_passthrough:u32,request:BtStage37Request,response_status:&mut u8,b:&mut B,
)->u32{
    let index=request.word16() as usize;
    if index>=b.index_count() as usize || !b.metadata_enabled(index){
        *response_status=66;
        return request_passthrough;
    }
    let object=b.lookup_object(request.key);
    if object==0{*response_status=2;return 0;}
    if b.object_byte223(object)&2==0{*response_status=26;return object;}
    let config=b.config_object(object);
    b.write_config_u16(config,4,request.word16());
    b.write_config_u16(config,2,request.word14());
    b.write_config_u8(config,11,1);
    b.commit_object(object)
}

/// Current `0x16C8E4` / legacy `sub_169DA4`.
///
/// Request byte +16 must be <=239. The current context must exist and have
/// byte +592 bit 0 set. The same object/byte+223 gate as `0x16C870` follows.
/// Success writes selector byte +16 at config +10, request word +14 at config
/// +2, clears config byte +11, and commits the object.
pub fn bt_stage37_selector_config<B:BtStage37ConfigBackend>(
    request:BtStage37Request,response_status:&mut u8,b:&mut B,
)->u32{
    let selector=request.byte16 as u32;
    if selector>0xEF{*response_status=18;return selector;}
    let context=b.current_context();
    if context==0{*response_status=66;return 0;}
    if b.context_byte592(context)&1==0{*response_status=18;return context;}
    let object=b.lookup_object(request.key);
    if object==0{*response_status=2;return 0;}
    if b.object_byte223(object)&2==0{*response_status=26;return object;}
    let config=b.config_object(object);
    b.write_config_u8(config,10,request.byte16);
    b.write_config_u16(config,2,request.word14());
    b.write_config_u8(config,11,0);
    b.commit_object(object)
}

/// Current `0x16C942` / legacy `sub_169E02`.
///
/// This variant has no metadata/context/object-flag gate. A successful lookup
/// writes request words +15/+17 to config +6/+8 and bytes +14/+19 to config
/// +12/+13. It returns the config-object result rather than calling commit.
pub fn bt_stage37_field_update<B:BtStage37ConfigBackend>(
    request:BtStage37Request,response_status:&mut u8,b:&mut B,
)->u32{
    let object=b.lookup_object(request.key);
    if object==0{*response_status=2;return 0;}
    let config=b.config_object(object);
    b.write_config_u16(config,6,request.word15());
    b.write_config_u16(config,8,request.word17());
    b.write_config_u8(config,12,request.byte14);
    b.write_config_u8(config,13,request.byte19);
    config
}

#[cfg(test)]
mod stage37_tests{
    use super::*;use std::vec::Vec;
    #[derive(Clone,Debug,PartialEq,Eq)]enum E{Ctx,CB592(u32),Lookup(u16),Obj223(u32),Cfg(u32),W16(u32,u8,u16),W8(u32,u8,u8),Commit(u32)}
    struct B{count:u8,enabled:[bool;4],ctx:u32,ctx592:u8,obj:u32,obj223:u8,cfg:u32,commit:u32,events:Vec<E>}
    impl Default for B{fn default()->Self{Self{count:4,enabled:[true;4],ctx:0xC000,ctx592:1,obj:0xD000,obj223:2,cfg:0xE000,commit:0xF000,events:Vec::new()}}}
    impl BtStage37ConfigBackend for B{
        fn index_count(&self)->u8{self.count}
        fn metadata_enabled(&self,i:usize)->bool{self.enabled.get(i).copied().unwrap_or(false)}
        fn current_context(&mut self)->u32{self.events.push(E::Ctx);self.ctx}
        fn context_byte592(&mut self,c:u32)->u8{self.events.push(E::CB592(c));self.ctx592}
        fn lookup_object(&mut self,k:u16)->u32{self.events.push(E::Lookup(k));self.obj}
        fn object_byte223(&mut self,o:u32)->u8{self.events.push(E::Obj223(o));self.obj223}
        fn config_object(&mut self,o:u32)->u32{self.events.push(E::Cfg(o));self.cfg}
        fn write_config_u16(&mut self,c:u32,o:u8,v:u16){self.events.push(E::W16(c,o,v))}
        fn write_config_u8(&mut self,c:u32,o:u8,v:u8){self.events.push(E::W8(c,o,v))}
        fn commit_object(&mut self,o:u32)->u32{self.events.push(E::Commit(o));self.commit}
    }
    fn r()->BtStage37Request{BtStage37Request{key:0x1234,byte14:0x78,byte15:0x56,byte16:2,byte17:0,byte18:0x9A,byte19:0xBC}}
    #[test]fn indexed_config_preserves_gates_writes_and_return_shapes(){
        let mut b=B::default();let mut s=0xAA;
        assert_eq!(bt_stage37_indexed_config(0x1111,r(),&mut s,&mut b),0xF000);assert_eq!(s,0xAA);
        assert_eq!(b.events,[E::Lookup(0x1234),E::Obj223(0xD000),E::Cfg(0xD000),E::W16(0xE000,4,2),E::W16(0xE000,2,0x5678),E::W8(0xE000,11,1),E::Commit(0xD000)]);
        let mut b=B{count:2,..B::default()};s=0;assert_eq!(bt_stage37_indexed_config(0x1111,r(),&mut s,&mut b),0x1111);assert_eq!(s,66);assert!(b.events.is_empty());
        let mut b=B{obj:0,..B::default()};s=0;assert_eq!(bt_stage37_indexed_config(0,r(),&mut s,&mut b),0);assert_eq!(s,2);
        let mut b=B{obj223:0,..B::default()};s=0;assert_eq!(bt_stage37_indexed_config(0,r(),&mut s,&mut b),0xD000);assert_eq!(s,26);
    }
    #[test]fn selector_config_preserves_context_object_gates_and_writes(){
        let mut b=B::default();let mut s=0x44;
        assert_eq!(bt_stage37_selector_config(r(),&mut s,&mut b),0xF000);assert_eq!(s,0x44);
        assert_eq!(b.events,[E::Ctx,E::CB592(0xC000),E::Lookup(0x1234),E::Obj223(0xD000),E::Cfg(0xD000),E::W8(0xE000,10,2),E::W16(0xE000,2,0x5678),E::W8(0xE000,11,0),E::Commit(0xD000)]);
        let mut q=r();q.byte16=240;let mut b=B::default();s=0;assert_eq!(bt_stage37_selector_config(q,&mut s,&mut b),240);assert_eq!(s,18);assert!(b.events.is_empty());
        let mut b=B{ctx:0,..B::default()};s=0;assert_eq!(bt_stage37_selector_config(r(),&mut s,&mut b),0);assert_eq!(s,66);
        let mut b=B{ctx592:0,..B::default()};s=0;assert_eq!(bt_stage37_selector_config(r(),&mut s,&mut b),0xC000);assert_eq!(s,18);
    }
    #[test]fn field_update_returns_config_and_preserves_exact_offsets(){
        let mut q=r();q.byte16=0x34;q.byte17=0x12;q.byte18=0xEF;let mut b=B::default();let mut s=7;
        assert_eq!(bt_stage37_field_update(q,&mut s,&mut b),0xE000);assert_eq!(s,7);
        assert_eq!(b.events,[E::Lookup(0x1234),E::Cfg(0xD000),E::W16(0xE000,6,0x3456),E::W16(0xE000,8,0xEF12),E::W8(0xE000,12,0x78),E::W8(0xE000,13,0xBC)]);
        let mut b=B{obj:0,..B::default()};s=0;assert_eq!(bt_stage37_field_update(r(),&mut s,&mut b),0);assert_eq!(s,2);
        assert_eq!(STAGE37_CURRENT_BT_INDEXED_CONFIG_ADDR,0x16C870);assert_eq!(STAGE37_CURRENT_BT_FIELD_UPDATE_ADDR,0x16C942);
    }
}

/// Stage 38: two current object-control routines recovered from globally unique
/// relocation-normalized complete-body matches. All unresolved runtime entries
/// remain explicit traits; no vendor names are inferred.
pub const STAGE38_CURRENT_BT_COMPARE_UPDATE_ADDR:u32=0x0016_C9A8;
pub const STAGE38_CURRENT_BT_FLAG_UPDATE_ADDR:u32=0x0016_CA4C;
pub const STAGE38_BT_LOOKUP_BOUNDARY:u32=0x0008_D34C;
pub const STAGE38_BT_PARSE_BOUNDARY:u32=0x0009_D3DC;
pub const STAGE38_BT_NOTIFY_BOUNDARY:u32=0x0008_6984;
pub const STAGE38_BT_FINALIZE_BOUNDARY:u32=0x0006_E774;
pub const STAGE38_BT_EQUAL_TAIL_BOUNDARY:u32=0x0008_4458;
pub const STAGE38_BT_FLAG_TAIL_BOUNDARY:u32=0x0008_4256;
pub const STAGE38_BT_GLOBAL_FLAGS_ADDR:u32=0x0020_30BC;
pub const STAGE38_BT_CONTROL_ADDR:u32=0x0020_807D;
pub const STAGE38_BT_CONTROL_PLUS29_ADDR:u32=0x0020_809A;
pub const STAGE38_BT_FALLBACK_ADDR:u32=0x0020_CE9C;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage38CompareRequest<'a>{
    pub key:u16,
    pub completion_id:u16,
    pub parse_payload:&'a[u8],
    pub mode_a:u8,
    pub mode_b:u8,
    pub replacement_word:u16,
}

pub trait BtStage38CompareBackend{
    type Handle:Copy;
    fn lookup(&mut self,key:u16)->Option<Self::Handle>;
    fn flags68(&mut self,h:Self::Handle)->u32;
    fn flags72(&mut self,h:Self::Handle)->u32;
    fn parse(&mut self,h:Self::Handle,payload:&[u8])->u32;
    fn write_word442(&mut self,h:Self::Handle,value:u16);
    fn word440(&mut self,h:Self::Handle)->u16;
    fn word444(&mut self,h:Self::Handle)->u16;
    fn mark_mismatch(&mut self,h:Self::Handle);
    fn notify_mismatch(&mut self,h:Self::Handle);
    fn finalize(&mut self,completion_id:u16,status:u32)->u32;
    fn equal_tail(&mut self,finalize_result:u32)->u32;
}

/// Safe control-flow model of current `0x16C9A8` / legacy `sub_169E68`.
///
/// The routine looks up an object by request key. Missing objects finalize with
/// status 2; either object bit `0x2000` gate finalizes with status 35. Otherwise
/// an opaque parser receives the request payload and the object field beginning
/// at firmware offset +440. When either request mode byte equals 4, the request
/// replacement word is stored at object +442 even when the parser later returns
/// an error. On parser success, unequal object words +440/+444 set the exact
/// local mismatch bits (+460|=2, +62|=8, +72|=0x2000), invoke the opaque update
/// boundary, and finalize with status zero. Equal words finalize with status zero
/// and then tail through the separate current boundary at `0x84458`.
pub fn bt_stage38_compare_update<B:BtStage38CompareBackend>(
    req:&BtStage38CompareRequest<'_>,b:&mut B,
)->u32{
    let Some(h)=b.lookup(req.key) else{return b.finalize(req.completion_id,2)};
    if b.flags68(h)&0x2000!=0 || b.flags72(h)&0x2000!=0{
        return b.finalize(req.completion_id,35);
    }
    let status=b.parse(h,req.parse_payload);
    if req.mode_a==4 || req.mode_b==4{b.write_word442(h,req.replacement_word);}
    if status!=0{return b.finalize(req.completion_id,status);}
    if b.word440(h)!=b.word444(h){
        b.mark_mismatch(h);
        b.notify_mismatch(h);
        return b.finalize(req.completion_id,0);
    }
    let result=b.finalize(req.completion_id,0);
    b.equal_tail(result)
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage38FlagRequest{pub key:u16,pub completion_id:u16}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum BtStage38EarlyTail{MissingObject,DeferredByObjectByte253}

pub trait BtStage38FlagBackend{
    type Handle:Copy;
    fn lookup(&mut self,key:u16)->Option<Self::Handle>;
    fn byte61(&mut self,h:Self::Handle)->u8;
    fn or_byte61(&mut self,h:Self::Handle,mask:u8);
    fn or_word72(&mut self,h:Self::Handle,mask:u32);
    fn byte253(&mut self,h:Self::Handle)->u8;
    fn global_flags(&mut self)->u16;
    fn control_byte(&mut self)->u8;
    fn control_plus29_byte(&mut self)->u8;
    fn fallback_byte(&mut self)->u8;
    fn notify_flags(&mut self,h:Self::Handle,control_hi:u32,selected_hi:u32);
    /// Models the two direct jump-outs to current `0x6E774` whose transient
    /// register ABI is intentionally not assigned a stronger source contract.
    fn early_finalize_tail(&mut self,req:BtStage38FlagRequest,reason:BtStage38EarlyTail)->u32;
    fn finalize(&mut self,completion_id:u16,status:u32)->u32;
    fn flag_tail(&mut self,finalize_result:u32)->u32;
}

/// Safe local-semantics model of current `0x16CA4C` / legacy `sub_169F0C`.
///
/// The runtime object lookup and both early jump-out ABI shapes remain opaque.
/// For a present object, byte +61 bit `0x20` skips directly to normal finalize.
/// Global flags `0x1010` force byte +61 bit 2. Otherwise the control byte at
/// current `0x20807D` chooses either current `0x20809A` (when bit 3 is set) or
/// fallback `0x20CE9C`. If the selected byte lacks bit 3 while object byte +253
/// is nonzero, firmware takes the opaque early finalize tail. The update path
/// sets object byte +61 bit 2 and word +72 bit 3, then calls the opaque notify
/// boundary with the control/selected bytes shifted into bits 28..31. Normal
/// finalize uses status zero; if object byte +61 bit `0x20` is set afterwards,
/// the finalize result tail-dispatches to current `0x84256`.
pub fn bt_stage38_flag_update<B:BtStage38FlagBackend>(
    req:BtStage38FlagRequest,b:&mut B,
)->u32{
    let Some(h)=b.lookup(req.key) else{
        return b.early_finalize_tail(req,BtStage38EarlyTail::MissingObject);
    };
    if b.byte61(h)&0x20==0{
        if b.global_flags()&0x1010==0x1010{
            b.or_byte61(h,4);
        }else{
            let control=b.control_byte();
            let selected=if control&8!=0{b.control_plus29_byte()}else{b.fallback_byte()};
            if selected&8==0 && b.byte253(h)!=0{
                return b.early_finalize_tail(req,BtStage38EarlyTail::DeferredByObjectByte253);
            }
            b.or_byte61(h,4);
            b.or_word72(h,8);
            b.notify_flags(h,(control as u32)<<28,(selected as u32)<<28);
        }
    }
    let result=b.finalize(req.completion_id,0);
    if b.byte61(h)&0x20!=0{b.flag_tail(result)}else{result}
}

#[cfg(test)]
mod stage38_tests{
    use super::*;use std::vec::Vec;
    #[derive(Clone,Copy)]struct Obj{f68:u32,f72:u32,b61:u8,b62:u8,b253:u8,b460:u8,w440:u16,w442:u16,w444:u16}
    impl Default for Obj{fn default()->Self{Self{f68:0,f72:0,b61:0,b62:0,b253:0,b460:0,w440:7,w442:0,w444:7}}}
    struct C{obj:Option<Obj>,parse:u32,events:Vec<u32>}
    impl BtStage38CompareBackend for C{
        type Handle=u8;fn lookup(&mut self,_:u16)->Option<u8>{self.obj.map(|_|0)}
        fn flags68(&mut self,_:u8)->u32{self.obj.unwrap().f68}fn flags72(&mut self,_:u8)->u32{self.obj.unwrap().f72}
        fn parse(&mut self,_:u8,_:&[u8])->u32{self.events.push(1);self.parse}
        fn write_word442(&mut self,_:u8,v:u16){self.obj.as_mut().unwrap().w442=v;self.events.push(2)}
        fn word440(&mut self,_:u8)->u16{self.obj.unwrap().w440}fn word444(&mut self,_:u8)->u16{self.obj.unwrap().w444}
        fn mark_mismatch(&mut self,_:u8){let o=self.obj.as_mut().unwrap();o.b460|=2;o.b62|=8;o.f72|=0x2000;self.events.push(3)}
        fn notify_mismatch(&mut self,_:u8){self.events.push(4)}
        fn finalize(&mut self,id:u16,s:u32)->u32{self.events.push(0x10000|s);id as u32+s}
        fn equal_tail(&mut self,r:u32)->u32{self.events.push(5);r+1000}
    }
    #[test]fn compare_update_preserves_gates_store_order_and_equal_tail(){
        let req=BtStage38CompareRequest{key:1,completion_id:9,parse_payload:&[1,2],mode_a:4,mode_b:0,replacement_word:0x1234};
        let mut c=C{obj:Some(Obj::default()),parse:0,events:Vec::new()};assert_eq!(bt_stage38_compare_update(&req,&mut c),1009);assert_eq!(c.obj.unwrap().w442,0x1234);assert_eq!(c.events,[1,2,0x10000,5]);
        let mut o=Obj::default();o.w444=8;let mut c=C{obj:Some(o),parse:0,events:Vec::new()};assert_eq!(bt_stage38_compare_update(&req,&mut c),9);let o=c.obj.unwrap();assert_eq!((o.b460,o.b62,o.f72),(2,8,0x2000));assert_eq!(c.events,[1,2,3,4,0x10000]);
        let mut o=Obj::default();o.f68=0x2000;let mut c=C{obj:Some(o),parse:0,events:Vec::new()};assert_eq!(bt_stage38_compare_update(&req,&mut c),44);assert_eq!(c.events,[0x10000|35]);
    }
    struct F{obj:Option<Obj>,g:u16,ctl:u8,p29:u8,fb:u8,events:Vec<u32>}
    impl BtStage38FlagBackend for F{
        type Handle=u8;fn lookup(&mut self,_:u16)->Option<u8>{self.obj.map(|_|0)}fn byte61(&mut self,_:u8)->u8{self.obj.unwrap().b61}
        fn or_byte61(&mut self,_:u8,m:u8){self.obj.as_mut().unwrap().b61|=m;self.events.push(1)}fn or_word72(&mut self,_:u8,m:u32){self.obj.as_mut().unwrap().f72|=m;self.events.push(2)}
        fn byte253(&mut self,_:u8)->u8{self.obj.unwrap().b253}fn global_flags(&mut self)->u16{self.g}fn control_byte(&mut self)->u8{self.ctl}
        fn control_plus29_byte(&mut self)->u8{self.p29}fn fallback_byte(&mut self)->u8{self.fb}
        fn notify_flags(&mut self,_:u8,a:u32,b:u32){self.events.push(0x30000000|((a>>28)<<8)|(b>>28))}
        fn early_finalize_tail(&mut self,_:BtStage38FlagRequest,r:BtStage38EarlyTail)->u32{self.events.push(match r{BtStage38EarlyTail::MissingObject=>6,BtStage38EarlyTail::DeferredByObjectByte253=>7});77}
        fn finalize(&mut self,id:u16,_:u32)->u32{self.events.push(8);id as u32}fn flag_tail(&mut self,r:u32)->u32{self.events.push(9);r+100}
    }
    #[test]fn flag_update_preserves_selection_defer_and_post_finalize_tail(){
        let req=BtStage38FlagRequest{key:2,completion_id:11};let mut f=F{obj:Some(Obj::default()),g:0,ctl:8,p29:8,fb:0,events:Vec::new()};assert_eq!(bt_stage38_flag_update(req,&mut f),11);assert_eq!((f.obj.unwrap().b61,f.obj.unwrap().f72),(4,8));assert_eq!(f.events,[1,2,0x30000808,8]);
        let mut o=Obj::default();o.b253=1;let mut f=F{obj:Some(o),g:0,ctl:0,p29:8,fb:0,events:Vec::new()};assert_eq!(bt_stage38_flag_update(req,&mut f),77);assert_eq!(f.events,[7]);
        let mut o=Obj::default();o.b61=0x20;let mut f=F{obj:Some(o),g:0,ctl:0,p29:0,fb:0,events:Vec::new()};assert_eq!(bt_stage38_flag_update(req,&mut f),111);assert_eq!(f.events,[8,9]);
        let mut f=F{obj:None,g:0,ctl:0,p29:0,fb:0,events:Vec::new()};assert_eq!(bt_stage38_flag_update(req,&mut f),77);assert_eq!(f.events,[6]);
    }
}

/// Stage 39: current request helper plus gated lookup/dispatch routine recovered
/// from globally unique relocation-normalized complete-body matches.
pub const STAGE39_CURRENT_BT_ENTRY_HELPER_ADDR:u32=0x0016_CAF0;
pub const STAGE39_CURRENT_BT_GATED_DISPATCH_ADDR:u32=0x0016_CB78;
pub const STAGE39_BT_ENTRY_STRIDE:u32=676;
pub const STAGE39_BT_ENTRY_BASE_PTR_ADDR:u32=0x0020_BE7C;
pub const STAGE39_BT_STACK_GUARD_WORD_ADDR:u32=0x0020_0890;
pub const STAGE39_BT_GATE_GLOBAL_ADDR:u32=0x0020_B0F0;
pub const STAGE39_BT_CALLBACK_THUMB:u32=0x0016_D05D;
pub const STAGE39_BT_ENTRY_PRECHECK_BOUNDARY:u32=0x0009_D58C;
pub const STAGE39_BT_INDEX_BOUNDARY:u32=0x0008_E450;
pub const STAGE39_BT_ENTRY_UPDATE_BOUNDARY:u32=0x0016_4444;
pub const STAGE39_BT_LOOKUP_BOUNDARY:u32=0x0003_3730;
pub const STAGE39_BT_GATE_BOUNDARY:u32=0x0004_F99C;
pub const STAGE39_BT_VALIDATE_BOUNDARY:u32=0x0006_595C;
pub const STAGE39_BT_FINALIZE_BOUNDARY:u32=0x0006_E774;
pub const STAGE39_BT_SUCCESS_BOUNDARY:u32=0x0003_2EA4;
pub const STAGE39_BT_STACK_GUARD_FAIL:u32=0x0000_94C0;
pub const STAGE39_BT_STAGE35_MATCH_ADDR:u32=0x0016_D490;

pub trait BtStage39EntryBackend{
    fn precheck(&mut self)->u32;
    fn map_index(&mut self,selector:u8)->u32;
    fn entry_base(&mut self)->u32;
    fn update_entry(&mut self,entry_plus40:u32,signed_value:i8)->u32;
    fn entry_byte83(&mut self,entry:u32)->u8;
}

/// Current `0x16CAF0` / legacy `sub_169FB0`.
/// The opaque precheck always runs. Only a zero caller-owned status byte enters
/// the indexed entry path: current entry address is `base + 676*map(selector)`,
/// the opaque update receives entry+40 and request byte +31 as signed i8, and
/// response byte +6 is then copied from entry byte +83.
pub fn bt_stage39_entry_helper<B:BtStage39EntryBackend>(
    selector:u8,signed_byte31:i8,response_status:u8,response_byte6:&mut u8,b:&mut B,
)->u32{
    let mut result=b.precheck();
    if response_status==0{
        let index=b.map_index(selector);
        let entry=b.entry_base().wrapping_add(STAGE39_BT_ENTRY_STRIDE.wrapping_mul(index));
        result=b.update_entry(entry.wrapping_add(40),signed_byte31);
        *response_byte6=b.entry_byte83(entry);
    }
    result
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage39DispatchRequest<'a>{
    pub key:u16,
    pub completion_id:u16,
    /// Bytes beginning at the firmware request field corresponding to `a1+9`.
    /// They remain opaque because current `0x6595C` consumes the original pointer.
    pub request_tail:&'a[u8],
}

pub trait BtStage39DispatchBackend{
    type Handle:Copy;
    fn lookup_kind3(&mut self,key:u16)->Option<Self::Handle>;
    /// Represents the exact short-circuit conjunction `*0x20B0F0 != 0 && 0x4F99C(...) != 0`
    /// without strengthening the still-opaque call ABI.
    fn global_gate_rejects(&mut self,h:Self::Handle)->bool;
    fn object_byte28(&mut self,h:Self::Handle)->u8;
    /// Existing Stage-35 current predicate at `0x16D490`.
    fn stage35_three_record_match(&mut self,h:Self::Handle)->bool;
    /// Current `0x6595C`, which receives the request pointer at +9 and a pointer
    /// to the local handle slot; returning the possibly replaced handle preserves
    /// that observable in/out behavior without naming the runtime contract.
    fn validate(&mut self,req:&BtStage39DispatchRequest<'_>,h:Self::Handle)->(u32,Self::Handle);
    fn finalize(&mut self,completion_id:u16,status:u32)->u32;
    fn success_followup(&mut self,h:Self::Handle,req:&BtStage39DispatchRequest<'_>,callback_thumb:u32)->u32;
}

/// Safe local-semantics model of current `0x16CB78` / legacy `sub_16A038`.
/// Missing lookup returns status 2. Present objects return status 12 when the
/// global/runtime gate rejects, object byte +28 masked by `0xF8` equals `0x68`,
/// or the already recovered Stage-35 three-record predicate matches. Otherwise
/// current `0x6595C` supplies status and may replace the local handle. Firmware
/// always calls the normal finalizer with the request completion id and status;
/// only status zero then calls current `0x32EA4` with the resulting handle,
/// original request pointer +9, and exact callback Thumb address `0x16D05D`.
/// Compiler stack-canary plumbing to `0x94C0` is intentionally omitted.
pub fn bt_stage39_gated_dispatch<B:BtStage39DispatchBackend>(
    req:&BtStage39DispatchRequest<'_>,b:&mut B,
)->u32{
    let (status,h)=match b.lookup_kind3(req.key){
        None=>(2,None),
        Some(h)=>{
            if b.global_gate_rejects(h) || b.object_byte28(h)&0xF8==0x68 || b.stage35_three_record_match(h){
                (12,Some(h))
            }else{
                let (s,new_h)=b.validate(req,h);
                (s,Some(new_h))
            }
        }
    };
    let result=b.finalize(req.completion_id,status);
    if status==0{
        b.success_followup(h.expect("status zero requires a validated handle"),req,STAGE39_BT_CALLBACK_THUMB)
    }else{result}
}

#[cfg(test)]
mod stage39_tests{
    use super::*;use std::vec::Vec;
    struct E{pre:u32,index:u32,base:u32,byte83:u8,calls:Vec<(u32,i32)>}
    impl BtStage39EntryBackend for E{
        fn precheck(&mut self)->u32{self.calls.push((1,0));self.pre}fn map_index(&mut self,_:u8)->u32{self.index}
        fn entry_base(&mut self)->u32{self.base}fn update_entry(&mut self,a:u32,v:i8)->u32{self.calls.push((a,v as i32));0x55}
        fn entry_byte83(&mut self,_:u32)->u8{self.byte83}
    }
    #[test]fn entry_helper_preserves_status_gate_stride_and_signed_byte(){
        let mut e=E{pre:7,index:2,base:0x1000,byte83:0xA5,calls:Vec::new()};let mut out=0;
        assert_eq!(bt_stage39_entry_helper(3,-2,0,&mut out,&mut e),0x55);assert_eq!(out,0xA5);assert_eq!(e.calls,[(1,0),(0x1000+2*676+40,-2)]);
        let mut e=E{pre:9,index:5,base:0x2000,byte83:1,calls:Vec::new()};out=4;assert_eq!(bt_stage39_entry_helper(2,127,18,&mut out,&mut e),9);assert_eq!(out,4);assert_eq!(e.calls,[(1,0)]);
    }
    struct D{lookup:bool,gate:bool,b28:u8,matches:bool,validate_status:u32,calls:Vec<u32>}
    impl BtStage39DispatchBackend for D{
        type Handle=u32;fn lookup_kind3(&mut self,_:u16)->Option<u32>{self.calls.push(1);if self.lookup{Some(10)}else{None}}
        fn global_gate_rejects(&mut self,_:u32)->bool{self.calls.push(2);self.gate}fn object_byte28(&mut self,_:u32)->u8{self.calls.push(3);self.b28}
        fn stage35_three_record_match(&mut self,_:u32)->bool{self.calls.push(4);self.matches}
        fn validate(&mut self,_:&BtStage39DispatchRequest<'_>,_:u32)->(u32,u32){self.calls.push(5);(self.validate_status,20)}
        fn finalize(&mut self,id:u16,s:u32)->u32{self.calls.push(0x100+s);id as u32+s}
        fn success_followup(&mut self,h:u32,_:&BtStage39DispatchRequest<'_>,cb:u32)->u32{self.calls.push(6);assert_eq!((h,cb),(20,0x16D05D));0x9999}
    }
    #[test]fn gated_dispatch_preserves_short_circuit_statuses_and_zero_followup(){
        let req=BtStage39DispatchRequest{key:3,completion_id:9,request_tail:&[1,2,3]};
        let mut d=D{lookup:false,gate:false,b28:0,matches:false,validate_status:0,calls:Vec::new()};assert_eq!(bt_stage39_gated_dispatch(&req,&mut d),11);assert_eq!(d.calls,[1,0x102]);
        let mut d=D{lookup:true,gate:true,b28:0,matches:false,validate_status:0,calls:Vec::new()};assert_eq!(bt_stage39_gated_dispatch(&req,&mut d),21);assert_eq!(d.calls,[1,2,0x10C]);
        let mut d=D{lookup:true,gate:false,b28:0x68,matches:false,validate_status:0,calls:Vec::new()};assert_eq!(bt_stage39_gated_dispatch(&req,&mut d),21);assert_eq!(d.calls,[1,2,3,0x10C]);
        let mut d=D{lookup:true,gate:false,b28:0,matches:false,validate_status:0,calls:Vec::new()};assert_eq!(bt_stage39_gated_dispatch(&req,&mut d),0x9999);assert_eq!(d.calls,[1,2,3,4,5,0x100,6]);
    }
}

/// Stage 40: current 442-byte request-selection transaction at `0x16CC00`,
/// recovered from a globally unique relocation-normalized complete-body match.
pub const STAGE40_CURRENT_BT_REQUEST_SELECT_ADDR:u32=0x0016_CC00;
pub const STAGE40_BT_LOOKUP_BOUNDARY:u32=0x0003_3730;
pub const STAGE40_BT_NULL_OUTPUT_BOUNDARY:u32=0x000B_DDBC;
pub const STAGE40_BT_GLOBAL_GATE_BOUNDARY:u32=0x0004_F99C;
pub const STAGE40_BT_STAGE35_MATCH_ADDR:u32=0x0016_D490;
pub const STAGE40_BT_MODE_BOUNDARY:u32=0x0003_3B8C;
pub const STAGE40_BT_TRANSFORM_BOUNDARY:u32=0x000B_0630;
pub const STAGE40_BT_OBJECT_PREDICATE_BOUNDARY:u32=0x0003_3D28;
pub const STAGE40_BT_SECONDARY_LOOKUP_BOUNDARY:u32=0x0003_35AC;
pub const STAGE40_BT_STACK_GUARD_FAIL:u32=0x0000_94C0;
pub const STAGE40_BT_STACK_GUARD_WORD_ADDR:u32=0x0020_0890;
pub const STAGE40_BT_GATE_GLOBAL_ADDR:u32=0x0020_B0F0;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage40Request{
    pub lookup_key:u16,
    pub field5:u32,
    pub field9:u32,
    pub field13:u16,
    pub field15:u16,
    pub field17:u8,
    pub field18:u16,
}

pub trait BtStage40Backend{
    type Handle:Copy+core::fmt::Debug+PartialEq+Eq;
    fn lookup_kind3(&mut self,key:u16)->Option<Self::Handle>;
    /// Exact short-circuit conjunction rooted at current global `0x20B0F0`
    /// and opaque current `0x4F99C`, without strengthening its ABI.
    fn global_gate_rejects(&mut self,h:Self::Handle)->bool;
    fn stage35_three_record_match(&mut self,h:Self::Handle)->bool;
    fn object_word0(&mut self,h:Self::Handle)->u32;
    fn object_byte256(&mut self,h:Self::Handle)->u8;
    fn object_byte167(&mut self,h:Self::Handle)->u8;
    fn object_byte215(&mut self,h:Self::Handle)->u8;
    fn object_word208(&mut self,h:Self::Handle)->u16;
    fn object_dword324(&mut self,h:Self::Handle)->u32;
    fn object_dword328(&mut self,h:Self::Handle)->u32;
    fn object_dword340(&mut self,h:Self::Handle)->u32;
    fn object_dword344(&mut self,h:Self::Handle)->u32;
    fn object_byte230(&mut self,h:Self::Handle)->u8;
    fn mode(&mut self)->u32;
    /// Current `0xB0630`: returns an 8-bit status plus the output word written
    /// through its third argument.
    fn transform(&mut self,input:u16,h:Self::Handle)->(u8,u16);
    fn object_predicate(&mut self,h:Self::Handle)->bool;
    fn secondary_lookup(&mut self,key:u8)->Option<Self::Handle>;
    /// Models the unconstrained stack word that current firmware copies to
    /// request field +18 when the secondary lookup is null and `0xB0630` is not
    /// called. This preserves the observed uninitialized-local edge explicitly.
    fn uninitialized_word_seed(&mut self)->u16;
}

/// Safe state-machine model of current `0x16CC00` / legacy `sub_16A0C0`.
///
/// `override_mode` is the firmware's second argument. The safe API requires a
/// non-null output slot; the original null-output path reaches opaque boundary
/// `0xBDDBC` and then dereferences the pointer, so no stronger behavior is
/// invented here. `selected` is cleared before all normal decision logic.
pub fn bt_stage40_request_select<B:BtStage40Backend>(
    req:&mut BtStage40Request,override_mode:bool,selected:&mut Option<B::Handle>,b:&mut B,
)->u32{
    let original_field5=req.field5;
    let original_field9=req.field9;
    let field15=req.field15;
    let mut transformed=req.field18;
    let Some(primary)=b.lookup_kind3(req.lookup_key) else{*selected=None;return 2};
    let (selector,kind_byte)=if override_mode{(4u32,0xFFu8)}else{(req.field13 as u32,req.field17)};
    *selected=None;

    if b.global_gate_rejects(primary)
        || b.stage35_three_record_match(primary)
        || (b.object_word0(primary).wrapping_sub(24)>2 && b.object_byte256(primary)&4!=0)
    {return 12;}

    if !override_mode{
        if selector<=3 || kind_byte.wrapping_sub(3)<=0xFB{return 18;}
        if transformed==0xFFFF{transformed=63;}
        transformed^=0x03C0;
        req.field18=transformed;
    }

    let mode=b.mode();
    if mode==1{
        let (status,out_word)=b.transform(transformed,primary);
        req.field18=out_word;
        if b.object_predicate(primary)
            || b.object_byte167(primary)&0x10==0
            || req.field18&0x03F8!=0
        {return 14;}
        *selected=Some(primary);
        return status as u32;
    }
    if mode==0 && !override_mode{return 12;}
    if mode!=0 && mode!=2{return 0;}

    let secondary_key=b.object_byte215(primary);
    let secondary=b.secondary_lookup(secondary_key);
    let (mut status,out_word)=match secondary{
        Some(h)=>{let (s,w)=b.transform(transformed,h);(s as u32,w)}
        None=>(0,b.uninitialized_word_seed()),
    };
    req.field18=out_word;

    if let Some(h)=secondary{
        if b.object_predicate(h) && b.object_byte167(h)&0x10!=0 && req.field18&0x03F8==0{return 14;}
    }

    let primary_matches=override_mode || (
        (b.object_word208(primary)&0x0FFF)==field15
        && (b.object_dword324(primary)==u32::MAX || original_field5==u32::MAX || original_field5==b.object_dword340(primary))
        && (b.object_dword328(primary)==u32::MAX || original_field9==u32::MAX || original_field9==b.object_dword344(primary))
    );
    if primary_matches{
        if status==0 && b.object_byte230(primary)&0x40!=0{status=35;}
    }else{status=18;}
    *selected=secondary;
    status
}

#[cfg(test)]
mod stage40_tests{
    use super::*;use std::vec::Vec;
    #[derive(Clone,Copy,Debug,PartialEq,Eq)]struct O(u8);
    struct B{lookup:Option<O>,gate:bool,match3:bool,word0:u32,b256:u8,b167:[u8;2],b215:u8,w208:u16,d324:u32,d328:u32,d340:u32,d344:u32,b230:u8,mode:u32,secondary:Option<O>,transform:(u8,u16),seed:u16,calls:Vec<u8>}
    impl BtStage40Backend for B{
        type Handle=O;fn lookup_kind3(&mut self,_:u16)->Option<O>{self.calls.push(1);self.lookup}fn global_gate_rejects(&mut self,_:O)->bool{self.calls.push(2);self.gate}
        fn stage35_three_record_match(&mut self,_:O)->bool{self.calls.push(3);self.match3}fn object_word0(&mut self,_:O)->u32{self.word0}fn object_byte256(&mut self,_:O)->u8{self.b256}
        fn object_byte167(&mut self,h:O)->u8{self.b167[h.0 as usize]}fn object_byte215(&mut self,_:O)->u8{self.b215}fn object_word208(&mut self,_:O)->u16{self.w208}
        fn object_dword324(&mut self,_:O)->u32{self.d324}fn object_dword328(&mut self,_:O)->u32{self.d328}fn object_dword340(&mut self,_:O)->u32{self.d340}fn object_dword344(&mut self,_:O)->u32{self.d344}
        fn object_byte230(&mut self,_:O)->u8{self.b230}fn mode(&mut self)->u32{self.calls.push(4);self.mode}fn transform(&mut self,_:u16,_:O)->(u8,u16){self.calls.push(5);self.transform}
        fn object_predicate(&mut self,_:O)->bool{self.calls.push(6);false}fn secondary_lookup(&mut self,_:u8)->Option<O>{self.calls.push(7);self.secondary}fn uninitialized_word_seed(&mut self)->u16{self.calls.push(8);self.seed}
    }
    fn base()->B{B{lookup:Some(O(0)),gate:false,match3:false,word0:24,b256:0,b167:[0x10,0],b215:1,w208:0x123,d324:u32::MAX,d328:u32::MAX,d340:0,d344:0,b230:0,mode:3,secondary:None,transform:(7,0),seed:0xBEEF,calls:Vec::new()}}
    fn req()->BtStage40Request{BtStage40Request{lookup_key:1,field5:10,field9:20,field13:4,field15:0x123,field17:0,field18:0xFFFF}}
    #[test]fn early_gates_and_nonoverride_transform_are_exact(){
        let mut b=base();b.lookup=None;let mut r=req();let mut out=None;assert_eq!(bt_stage40_request_select(&mut r,false,&mut out,&mut b),2);
        let mut b=base();b.gate=true;let mut r=req();assert_eq!(bt_stage40_request_select(&mut r,false,&mut out,&mut b),12);
        let mut b=base();let mut r=req();r.field13=3;assert_eq!(bt_stage40_request_select(&mut r,false,&mut out,&mut b),18);
        let mut b=base();b.mode=3;let mut r=req();assert_eq!(bt_stage40_request_select(&mut r,false,&mut out,&mut b),0);assert_eq!(r.field18,0x03FF);
    }
    #[test]fn mode1_success_and_rejection_preserve_selected_and_status(){
        let mut b=base();b.mode=1;b.transform=(9,0);let mut r=req();let mut out=None;assert_eq!(bt_stage40_request_select(&mut r,false,&mut out,&mut b),9);assert_eq!(out,Some(O(0)));assert_eq!(r.field18,0);
        let mut b=base();b.mode=1;b.transform=(9,0x3F8);let mut r=req();let mut out=None;assert_eq!(bt_stage40_request_select(&mut r,false,&mut out,&mut b),14);assert_eq!(out,None);
    }
    #[test]fn secondary_path_preserves_uninitialized_seed_match_rules_and_status35(){
        let mut b=base();b.mode=2;b.secondary=None;b.seed=0x55AA;b.b230=0x40;let mut r=req();let mut out=None;assert_eq!(bt_stage40_request_select(&mut r,false,&mut out,&mut b),35);assert_eq!(r.field18,0x55AA);assert_eq!(out,None);assert!(b.calls.contains(&8));
        let mut b=base();b.mode=2;b.secondary=Some(O(1));b.transform=(5,0);b.d324=11;b.d340=12;let mut r=req();let mut out=None;assert_eq!(bt_stage40_request_select(&mut r,false,&mut out,&mut b),18);assert_eq!(out,Some(O(1)));
        let mut b=base();b.mode=0;b.secondary=Some(O(1));b.transform=(4,0);let mut r=req();let mut out=None;assert_eq!(bt_stage40_request_select(&mut r,true,&mut out,&mut b),4);assert_eq!(out,Some(O(1)));
    }
}

/// Stage 41: current 294-byte object/request update transaction at `0x16D05C`,
/// recovered from a globally unique relocation-normalized complete-body match.
pub const STAGE41_CURRENT_BT_OBJECT_UPDATE_ADDR:u32=0x0016_D05C;
pub const STAGE41_BT_LOOKUP_BOUNDARY:u32=0x0003_3730;
pub const STAGE41_BT_MISSING_BOUNDARY:u32=0x0006_F438;
pub const STAGE41_BT_MODE_BOUNDARY:u32=0x0003_3B8C;
pub const STAGE41_BT_OBJECT_PREDICATE_BOUNDARY:u32=0x0003_3D28;
pub const STAGE41_BT_RESOLVE_BOUNDARY:u32=0x0004_FF46;
pub const STAGE41_BT_NOTIFY_BOUNDARY:u32=0x0006_F340;
pub const STAGE41_BT_FOLLOWUP_BOUNDARY:u32=0x0003_2F08;
pub const STAGE41_BT_FINAL_BOUNDARY:u32=0x0005_4BE4;
pub const STAGE41_BT_STACK_GUARD_FAIL:u32=0x0000_94C0;
pub const STAGE41_BT_STACK_GUARD_WORD_ADDR:u32=0x0020_0890;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage41Request{
    pub opcode:u16,
    pub key:u16,
    pub dword5:u32,
    pub dword9:u32,
    pub byte57:u8,
    pub byte58:u8,
    pub word59:u16,
    pub byte61:u8,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct BtStage41History{
    pub dword81:u32,pub dword82:u32,pub dword83:u32,pub dword84:u32,
    pub dword89:u32,pub dword90:u32,pub dword91:u32,pub dword92:u32,
}

pub trait BtStage41Backend{
    type Handle:Copy+core::fmt::Debug+PartialEq+Eq;
    fn lookup_kind3(&mut self,key:u16)->Option<Self::Handle>;
    fn missing(&mut self,key:u16,status:u32)->u32;
    fn mode(&mut self,h:Self::Handle)->u32;
    /// Current `0x33D28`; its integer return is preserved because unsupported
    /// mode values return this result unchanged.
    fn object_predicate(&mut self,h:Self::Handle)->u32;
    fn object_byte167(&mut self,h:Self::Handle)->u8;
    fn object_dword28(&mut self,h:Self::Handle)->u32;
    fn object_byte166(&mut self,h:Self::Handle)->u8;
    fn resolve(&mut self,h:Self::Handle,req:&BtStage41Request)->(u32,Option<Self::Handle>);
    fn notify(&mut self,h:Self::Handle,candidate:Option<Self::Handle>,code:u32,is_opcode_1031:bool);
    fn followup(&mut self,h:Self::Handle,a:u32,b:u32)->u32;
    fn candidate_byte230(&mut self,h:Self::Handle)->u8;
    fn candidate_byte231(&mut self,h:Self::Handle)->u8;
    fn set_candidate_byte230(&mut self,h:Self::Handle,value:u8);
    fn set_candidate_byte231(&mut self,h:Self::Handle,value:u8);
    fn history(&mut self,h:Self::Handle)->BtStage41History;
    fn store_history(&mut self,h:Self::Handle,value:BtStage41History);
    fn set_primary_word166(&mut self,h:Self::Handle,value:u16);
    fn set_primary_byte336(&mut self,h:Self::Handle,value:u8);
    fn set_primary_word167(&mut self,h:Self::Handle,value:u16);
    fn final_call(&mut self,h:Self::Handle,kind:u32)->u32;
    /// The binary dereferences a null candidate if mode 1 produces code zero
    /// without populating the resolver output slot. Safe Rust makes that crash
    /// edge explicit instead of silently inventing an object.
    fn null_candidate_fault(&mut self)->!;
}

/// Safe local-semantics model of current `0x16D05C` / legacy `sub_16A4B4`.
/// Compiler stack-canary mechanics are omitted; all unresolved runtime calls
/// remain trait methods.
pub fn bt_stage41_object_update<B:BtStage41Backend>(req:&mut BtStage41Request,b:&mut B)->u32{
    let Some(primary)=b.lookup_kind3(req.key) else{return b.missing(req.key,2)};
    let mode=b.mode(primary);
    let predicate_result=b.object_predicate(primary);
    if predicate_result!=0 && b.object_byte167(primary)&0x10!=0{req.word59&=0xFFF8;}

    if mode==1{
        let (code,candidate)=if b.object_dword28(primary)&0xF8==0x68{
            (b.object_byte166(primary) as u32,None)
        }else{b.resolve(primary,req)};
        if code!=0{
            b.notify(primary,candidate,code,req.opcode==1031);
            return b.followup(primary,1,3);
        }
        let Some(h)=candidate else{b.null_candidate_fault()};
        let b231=b.candidate_byte231(h);
        let b230=b.candidate_byte230(h);
        b.set_candidate_byte230(h,(b230&0x7F)|((req.opcode==1031) as u8)<<7);
        b.set_candidate_byte231(h,(b231&0x7F)|((req.opcode==1085) as u8)<<7);
        return b.final_call(h,0);
    }

    if mode!=0 && mode!=2{return predicate_result;}
    let mut h=b.history(primary);
    h.dword89=h.dword81;h.dword90=h.dword82;h.dword91=h.dword83;h.dword92=h.dword84;
    h.dword81=req.dword5;h.dword82=req.dword9;
    b.store_history(primary,h);
    b.set_primary_word166(primary,(req.byte57 as u16)|((req.byte58 as u16)<<8));
    b.set_primary_byte336(primary,req.byte61);
    b.set_primary_word167(primary,req.word59);
    b.final_call(primary,6)
}

#[cfg(test)]
mod stage41_tests{
    use super::*;use std::vec::Vec;
    #[derive(Clone,Copy,Debug,PartialEq,Eq)]struct H(u8);
    struct B{lookup:Option<H>,mode:u32,pred:u32,b167:u8,d28:u32,b166:u8,res:(u32,Option<H>),b230:u8,b231:u8,hist:BtStage41History,calls:Vec<u32>}
    impl BtStage41Backend for B{
        type Handle=H;fn lookup_kind3(&mut self,_:u16)->Option<H>{self.calls.push(1);self.lookup}fn missing(&mut self,k:u16,s:u32)->u32{self.calls.push(2);k as u32+s}
        fn mode(&mut self,_:H)->u32{self.calls.push(3);self.mode}fn object_predicate(&mut self,_:H)->u32{self.calls.push(4);self.pred}fn object_byte167(&mut self,_:H)->u8{self.b167}
        fn object_dword28(&mut self,_:H)->u32{self.d28}fn object_byte166(&mut self,_:H)->u8{self.b166}fn resolve(&mut self,_:H,_:&BtStage41Request)->(u32,Option<H>){self.calls.push(5);self.res}
        fn notify(&mut self,_:H,_:Option<H>,c:u32,f:bool){self.calls.push(0x100+c+f as u32)}fn followup(&mut self,_:H,a:u32,b:u32)->u32{self.calls.push(6);a*10+b}
        fn candidate_byte230(&mut self,_:H)->u8{self.b230}fn candidate_byte231(&mut self,_:H)->u8{self.b231}fn set_candidate_byte230(&mut self,_:H,v:u8){self.b230=v;self.calls.push(7)}fn set_candidate_byte231(&mut self,_:H,v:u8){self.b231=v;self.calls.push(8)}
        fn history(&mut self,_:H)->BtStage41History{self.hist}fn store_history(&mut self,_:H,v:BtStage41History){self.hist=v;self.calls.push(9)}fn set_primary_word166(&mut self,_:H,v:u16){self.calls.push(0x10000+v as u32)}fn set_primary_byte336(&mut self,_:H,v:u8){self.calls.push(0x20000+v as u32)}fn set_primary_word167(&mut self,_:H,v:u16){self.calls.push(0x30000+v as u32)}
        fn final_call(&mut self,_:H,k:u32)->u32{self.calls.push(10+k);0x9000+k}fn null_candidate_fault(&mut self)->!{panic!("null candidate")}
    }
    fn req(op:u16)->BtStage41Request{BtStage41Request{opcode:op,key:4,dword5:0x11,dword9:0x22,byte57:0x33,byte58:0x44,word59:0xFFFF,byte61:0x55}}
    fn base()->B{B{lookup:Some(H(0)),mode:0,pred:0,b167:0,d28:0,b166:0,res:(0,Some(H(1))),b230:0xAA,b231:0xBB,hist:BtStage41History{dword81:1,dword82:2,dword83:3,dword84:4,dword89:0,dword90:0,dword91:0,dword92:0},calls:Vec::new()}}
    #[test]fn missing_mask_and_unsupported_mode_preserve_result(){
        let mut b=base();b.lookup=None;let mut r=req(0);assert_eq!(bt_stage41_object_update(&mut r,&mut b),6);
        let mut b=base();b.mode=7;b.pred=9;b.b167=0x10;let mut r=req(0);assert_eq!(bt_stage41_object_update(&mut r,&mut b),9);assert_eq!(r.word59,0xFFF8);
    }
    #[test]fn mode1_resolver_preserves_notify_and_opcode_bits(){
        let mut b=base();b.mode=1;b.res=(5,Some(H(1)));let mut r=req(1031);assert_eq!(bt_stage41_object_update(&mut r,&mut b),13);assert!(b.calls.contains(&(0x100+5+1)));
        let mut b=base();b.mode=1;let mut r=req(1031);assert_eq!(bt_stage41_object_update(&mut r,&mut b),0x9000);assert_eq!(b.b230,0xAA|0x80);assert_eq!(b.b231,0x3B);
        let mut b=base();b.mode=1;let mut r=req(1085);let _=bt_stage41_object_update(&mut r,&mut b);assert_eq!(b.b230,0x2A);assert_eq!(b.b231,0xBB|0x80);
    }
    #[test]fn mode0_or2_shifts_history_then_commits_request_fields(){
        let mut b=base();b.mode=2;let mut r=req(0);assert_eq!(bt_stage41_object_update(&mut r,&mut b),0x9006);assert_eq!((b.hist.dword89,b.hist.dword90,b.hist.dword91,b.hist.dword92),(1,2,3,4));assert_eq!((b.hist.dword81,b.hist.dword82),(0x11,0x22));assert!(b.calls.contains(&(0x10000+0x4433)));assert!(b.calls.contains(&(0x20000+0x55)));assert!(b.calls.contains(&(0x30000+0xFFFF)));
    }
}

/// Stage 42: current request/mode transaction at `0x16CDC4`, promoted from
/// the globally unique relocation-normalized legacy `sub_16A284` body.
pub const STAGE42_CURRENT_BT_REQUEST_MODE_ADDR:u32=0x0016_CDC4;
pub const STAGE42_CURRENT_BT_MODE1_POST_ADDR:u32=0x0016_D90C;
pub const STAGE42_BT_LOOKUP_BOUNDARY:u32=0x0003_3730;
pub const STAGE42_BT_MODE_BOUNDARY:u32=0x0003_3B8C;
pub const STAGE42_BT_VALIDATE_BOUNDARY:u32=0x0003_3AC8;
pub const STAGE42_BT_STATUS_BOUNDARY:u32=0x0006_E774;
pub const STAGE42_BT_MODE0_BUILD_BOUNDARY:u32=0x0006_5244;
pub const STAGE42_BT_MODE0_FOLLOWUP_BOUNDARY:u32=0x0003_2EA4;
pub const STAGE42_BT_GUARD_WORD_ADDR:u32=0x0020_0890;
pub const STAGE42_BT_TEMPLATE_CONTEXT_ADDR:u32=0x0020_8338;
pub const STAGE42_BT_MODE0_CALLBACK_THUMB:u32=0x0007_1C61;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage42Mode0Message{
    pub completion:u16,
    pub kind:u8,
    pub lookup_key:u16,
    pub byte16:u8,
    pub byte17:u8,
    pub byte18:u8,
    pub byte19:u8,
    pub byte20:u8,
    pub byte21:u8,
    pub byte22:u8,
    pub byte23:u8,
    pub template_word:u16,
    pub control_hi3:u16,
}

pub trait BtStage42Backend{
    /// Current `0x33730(key,3)`: return an opaque object handle when present.
    fn lookup_kind3(&mut self,key:u16)->Option<u32>;
    /// Current `0x33B8C(*object)` mode result.
    fn mode(&mut self,object:u32)->u32;
    /// Current `0x33AC8(mask)` boolean-like validation.
    fn validate_mask(&mut self,mask:u16)->bool;
    fn object_byte31(&mut self,object:u32)->u8;
    fn set_object_word236(&mut self,object:u32,value:u16);
    /// Current `0x6E774(completion,status)` pointer-shaped/integer result.
    fn status(&mut self,completion:u16,status:u32)->u32;
    /// Current relocated `0x16D90C(object)`; semantics remain a later closure.
    fn mode1_post(&mut self,object:u32)->u32;
    /// Current word loaded through literal context `0x208338` at byte offset 77.
    fn template_word77(&mut self)->u16;
    /// Current `0x65244(&message,1,&out_handle)`.
    fn build_mode0(&mut self,message:&BtStage42Mode0Message,out_handle:&mut u32)->u32;
    /// Current `0x32EA4(out_handle,&message,0x71C61)`.
    fn mode0_followup(&mut self,out_handle:u32,message:&BtStage42Mode0Message,callback_thumb:u32)->u32;
}

fn bt_stage42_u16(raw:&[u8;16],off:usize)->u16{u16::from_le_bytes([raw[off],raw[off+1]])}

/// Safe source-level model of current `0x16CDC4`.
///
/// The raw request layout is kept to preserve the firmware's unaligned u16 at
/// byte +9, lookup key at +12, and control word at +14.  The early
/// `(control ^ 0x3306) & ~1 == 0` path returns the opaque object handle without
/// status/finalization.  Mode 1 validates `(control ^ 0x3306) & 0xFF1E`, gates
/// on object byte +31 bit 3, stores the mask at object word +236, reports
/// status zero, then invokes the independently relocated current `0x16D90C`.
/// Mode 0 builds the exact constant-shaped local message and reports the build
/// result before the zero-only follow-up. Other modes preserve the opaque mode
/// result exactly.
pub fn bt_stage42_request_mode<B:BtStage42Backend>(request:&[u8;16],b:&mut B)->u32{
    let completion=bt_stage42_u16(request,9);
    let key=bt_stage42_u16(request,12);
    let control=bt_stage42_u16(request,14);
    let Some(object)=b.lookup_kind3(key) else{return b.status(completion,2)};
    let x=control^0x3306;
    if x&0xFFFE==0{return object;}
    let mode=b.mode(object);
    if mode==1{
        let mask=x&0xFF1E;
        if !b.validate_mask(mask){return b.status(completion,18);}
        let bit=b.object_byte31(object)&8;
        if bit!=0{return b.status(completion,12);}
        b.set_object_word236(object,mask);
        let _=b.status(completion,bit as u32);
        return b.mode1_post(object);
    }
    if mode==0{
        let message=BtStage42Mode0Message{
            completion,
            kind:17,
            lookup_key:key,
            byte16:64,
            byte17:31,
            byte18:0,
            byte19:0,
            byte20:64,
            byte21:31,
            byte22:0,
            byte23:0,
            template_word:b.template_word77(),
            control_hi3:((control as u8)>>5) as u16,
        };
        let mut out=0u32;
        let r=b.build_mode0(&message,&mut out);
        let status_result=b.status(completion,r);
        if r==0{return b.mode0_followup(out,&message,STAGE42_BT_MODE0_CALLBACK_THUMB);}
        return status_result;
    }
    mode
}

#[cfg(test)]
mod stage42_tests{
    use super::*;use std::vec::Vec;
    struct B{obj:Option<u32>,mode:u32,valid:bool,b31:u8,template:u16,build:u32,out:u32,calls:Vec<(u8,u32,u32)>,stored:Option<u16>}
    impl BtStage42Backend for B{
        fn lookup_kind3(&mut self,k:u16)->Option<u32>{self.calls.push((1,k as u32,3));self.obj}
        fn mode(&mut self,o:u32)->u32{self.calls.push((2,o,0));self.mode}
        fn validate_mask(&mut self,m:u16)->bool{self.calls.push((3,m as u32,0));self.valid}
        fn object_byte31(&mut self,_:u32)->u8{self.b31}
        fn set_object_word236(&mut self,_:u32,v:u16){self.stored=Some(v)}
        fn status(&mut self,c:u16,s:u32)->u32{self.calls.push((4,c as u32,s));0x8000_0000|s}
        fn mode1_post(&mut self,o:u32)->u32{self.calls.push((5,o,0));0x1111}
        fn template_word77(&mut self)->u16{self.template}
        fn build_mode0(&mut self,m:&BtStage42Mode0Message,out:&mut u32)->u32{assert_eq!(m.kind,17);*out=self.out;self.calls.push((6,m.lookup_key as u32,m.control_hi3 as u32));self.build}
        fn mode0_followup(&mut self,o:u32,_:&BtStage42Mode0Message,cb:u32)->u32{self.calls.push((7,o,cb));0x2222}
    }
    fn req(completion:u16,key:u16,control:u16)->[u8;16]{let mut r=[0u8;16];r[9..11].copy_from_slice(&completion.to_le_bytes());r[12..14].copy_from_slice(&key.to_le_bytes());r[14..16].copy_from_slice(&control.to_le_bytes());r}
    fn backend()->B{B{obj:Some(0x55),mode:1,valid:true,b31:0,template:0x1234,build:0,out:0x66,calls:Vec::new(),stored:None}}
    #[test]fn missing_early_mask_and_other_mode_preserve_returns(){
        let mut b=backend();b.obj=None;assert_eq!(bt_stage42_request_mode(&req(7,9,0),&mut b),0x8000_0002);
        let mut b=backend();assert_eq!(bt_stage42_request_mode(&req(1,2,0x3306),&mut b),0x55);assert_eq!(b.calls.len(),1);
        let mut b=backend();b.mode=7;assert_eq!(bt_stage42_request_mode(&req(1,2,0),&mut b),7);
    }
    #[test]fn mode1_preserves_validate_bit_gate_store_status_and_post(){
        let mut b=backend();let r=bt_stage42_request_mode(&req(0x1234,4,0),&mut b);assert_eq!(r,0x1111);assert_eq!(b.stored,Some(0x3306&0xFF1E));assert!(b.calls.iter().any(|x|*x==(4,0x1234,0)));assert!(b.calls.iter().any(|x|x.0==5));
        let mut b=backend();b.valid=false;assert_eq!(bt_stage42_request_mode(&req(3,4,0),&mut b),0x8000_0012);
        let mut b=backend();b.b31=8;assert_eq!(bt_stage42_request_mode(&req(3,4,0),&mut b),0x8000_000c);
    }
    #[test]fn mode0_message_constants_status_and_zero_followup_are_exact(){
        let mut b=backend();b.mode=0;b.template=0xABCD;b.out=0xCAFE;b.build=0;assert_eq!(bt_stage42_request_mode(&req(0x102,0x304,0xE123),&mut b),0x2222);assert!(b.calls.contains(&(4,0x102,0)));assert!(b.calls.contains(&(7,0xCAFE,STAGE42_BT_MODE0_CALLBACK_THUMB)));
        let mut b=backend();b.mode=0;b.build=5;assert_eq!(bt_stage42_request_mode(&req(8,9,0x8123),&mut b),0x8000_0005);assert!(!b.calls.iter().any(|x|x.0==7));
        assert_eq!(STAGE42_CURRENT_BT_REQUEST_MODE_ADDR,0x16CDC4);assert_eq!(STAGE42_CURRENT_BT_MODE1_POST_ADDR,0x16D90C);
    }
}

/// Stage 43: close the current Stage-42 mode-1 continuation at `0x16D90C`.
/// The body is the globally unique relocation-normalized current match of
/// legacy `sub_16AA10`. Runtime entries remain opaque trait boundaries.
pub const STAGE43_CURRENT_BT_MODE1_POST_ADDR:u32=0x0016_D90C;
pub const STAGE43_BT_PROBE_BOUNDARY:u32=0x0003_CCDC;
pub const STAGE43_BT_TRANSITION_BOUNDARY:u32=0x0003_CC9E;
pub const STAGE43_BT_UPDATE_BOUNDARY:u32=0x0004_BC44;
pub const STAGE43_BT_NOTIFY_BOUNDARY:u32=0x0006_F246;
pub const STAGE43_BT_FINAL_TAIL_BOUNDARY:u32=0x0004_14A0;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage43ObjectState{
    pub byte31:u8,
    pub word100:u16,
    pub word104:u16,
    pub byte167:u8,
    pub byte235:u8,
    pub word236:u16,
}

/// Register-level result of current `0x3CCDC`.
/// `r1_after` is intentionally exposed because one binary edge forwards the
/// caller-volatile R1 value produced by this call directly into `0x4BC44`.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage43ProbeResult{pub r0:u32,pub r1_after:u32}

pub trait BtStage43Backend{
    fn probe(&mut self,object:u32,word236:u16)->BtStage43ProbeResult;
    fn transition(&mut self,object:u32,arg1:u32)->u32;
    fn update(&mut self,object:u32,forwarded_r1:u32);
    fn notify(&mut self,zero:u32,word100:u16,delta:u16);
    fn final_tail(&mut self,object:u32,one:u32,zero:u32)->u32;
}

fn bt_stage43_commit<B:BtStage43Backend>(
    object:u32,state:&mut BtStage43ObjectState,forwarded_r1:u32,b:&mut B,
)->u32{
    state.word104=state.word236;
    b.update(object,forwarded_r1);
    b.notify(0,state.word100,state.word236^0x3306);
    b.final_tail(object,1,0)
}

/// Safe local-semantics model of current `0x16D90C` / legacy `sub_16AA10`.
///
/// The exact local masks and field transitions are preserved. The unusual
/// probe-zero path deliberately forwards the post-call R1 value from opaque
/// `0x3CCDC` into opaque `0x4BC44`; it is not reconstructed from `word236`.
pub fn bt_stage43_mode1_post<B:BtStage43Backend>(
    object:u32,state:&mut BtStage43ObjectState,b:&mut B,
)->u32{
    let word236=state.word236;
    let high=state.byte167&0xE0;
    if word236&0x3306==0{
        if high==0{
            state.byte31|=8;
            if state.byte235&0x30==0{return object;}
            return b.transition(object,0);
        }
        return bt_stage43_commit(object,state,high as u32,b);
    }
    if high==0{
        return bt_stage43_commit(object,state,word236 as u32,b);
    }
    let probe=b.probe(object,word236);
    if probe.r0==0{
        return bt_stage43_commit(object,state,probe.r1_after,b);
    }
    state.byte31|=8;
    if state.byte235&0x30==0x10{return probe.r0;}
    b.transition(object,1)
}

#[cfg(test)]
mod stage43_tests{
    use super::*;use std::vec::Vec;
    struct B{probe:BtStage43ProbeResult,events:Vec<(u8,u32,u32)>,transition_ret:u32,tail_ret:u32}
    impl BtStage43Backend for B{
        fn probe(&mut self,o:u32,w:u16)->BtStage43ProbeResult{self.events.push((1,o,w as u32));self.probe}
        fn transition(&mut self,o:u32,a:u32)->u32{self.events.push((2,o,a));self.transition_ret}
        fn update(&mut self,o:u32,r1:u32){self.events.push((3,o,r1))}
        fn notify(&mut self,z:u32,w:u16,d:u16){self.events.push((4,z,((w as u32)<<16)|d as u32))}
        fn final_tail(&mut self,o:u32,one:u32,zero:u32)->u32{self.events.push((5,o,(one<<16)|zero));self.tail_ret}
    }
    fn backend()->B{B{probe:BtStage43ProbeResult{r0:1,r1_after:0},events:Vec::new(),transition_ret:0x2222,tail_ret:0x3333}}
    fn state()->BtStage43ObjectState{BtStage43ObjectState{byte31:0x40,word100:0x1234,word104:0,byte167:0,byte235:0,word236:0}}
    #[test]fn zero_mask_paths_preserve_return_transition_and_forwarded_high_bits(){
        let mut b=backend();let mut s=state();assert_eq!(bt_stage43_mode1_post(0x55,&mut s,&mut b),0x55);assert_eq!(s.byte31,0x48);assert!(b.events.is_empty());
        let mut b=backend();let mut s=state();s.byte235=0x20;assert_eq!(bt_stage43_mode1_post(0x55,&mut s,&mut b),0x2222);assert_eq!(b.events,[(2,0x55,0)]);
        let mut b=backend();let mut s=state();s.byte167=0xA5;assert_eq!(bt_stage43_mode1_post(0x55,&mut s,&mut b),0x3333);assert_eq!(s.word104,0);assert_eq!(b.events[0],(3,0x55,0xA0));
    }
    #[test]fn nonzero_word_without_high_bits_forwards_original_word236(){
        let mut b=backend();let mut s=state();s.word236=0x3306;s.word100=0xBEEF;assert_eq!(bt_stage43_mode1_post(7,&mut s,&mut b),0x3333);assert_eq!(s.word104,0x3306);assert_eq!(b.events[0],(3,7,0x3306));assert_eq!(b.events[1],(4,0,0xBEEF0000));
    }
    #[test]fn probe_paths_preserve_volatile_r1_return_and_transition_gate(){
        let mut b=backend();b.probe=BtStage43ProbeResult{r0:0,r1_after:0xDEAD_BEEF};let mut s=state();s.word236=2;s.byte167=0xE0;assert_eq!(bt_stage43_mode1_post(9,&mut s,&mut b),0x3333);assert_eq!(b.events[0],(1,9,2));assert_eq!(b.events[1],(3,9,0xDEAD_BEEF));
        let mut b=backend();b.probe=BtStage43ProbeResult{r0:0x77,r1_after:0x11};let mut s=state();s.word236=2;s.byte167=0x20;s.byte235=0x10;assert_eq!(bt_stage43_mode1_post(9,&mut s,&mut b),0x77);assert_eq!(s.byte31,0x48);assert_eq!(b.events,[(1,9,2)]);
        let mut b=backend();b.probe=BtStage43ProbeResult{r0:0x77,r1_after:0x11};let mut s=state();s.word236=2;s.byte167=0x20;s.byte235=0x20;assert_eq!(bt_stage43_mode1_post(9,&mut s,&mut b),0x2222);assert_eq!(b.events,[(1,9,2),(2,9,1)]);
        assert_eq!(STAGE43_CURRENT_BT_MODE1_POST_ADDR,0x16D90C);
    }
}

/// Stage 44: adjacent current continuations after the recovered mode-1 post path.
///
/// Both entry points are promoted only after whole-current-code relocation-normalized
/// uniqueness was proven against the current Orange Pi BCM4362A2 PatchRAM image.
/// External runtime calls remain deliberately opaque traits.
pub const STAGE44_CURRENT_BT_POST_GATE_ADDR: u32 = 0x0016_D99C;
pub const STAGE44_CURRENT_BT_POST_SEQUENCE_ADDR: u32 = 0x0016_D9E8;
pub const STAGE44_BT_SHARED_FLAGS_BASE_ADDR: u32 = 0x0020_8830;
pub const STAGE44_BT_GLOBAL_MODE_ADDR: u32 = 0x0020_90CC;

pub const STAGE44_BT_PROBE_BOUNDARY: u32 = 0x0003_CCDC;
pub const STAGE44_BT_PROBE_FOLLOWUP_BOUNDARY: u32 = 0x0003_CC9E;
pub const STAGE44_BT_FLAGGED_TAIL_BOUNDARY: u32 = 0x0003_C3B0;
pub const STAGE44_BT_DEFAULT_TAIL_BOUNDARY: u32 = 0x0002_EC18;
pub const STAGE44_BT_MODE_WRITE_BOUNDARY: u32 = 0x0004_14A0;
pub const STAGE44_BT_BIT10_PREPARE_BOUNDARY: u32 = 0x0002_EB58;
pub const STAGE44_BT_FORWARD_BOUNDARY: u32 = 0x0006_E9E0;
pub const STAGE44_BT_STEP_A_BOUNDARY: u32 = 0x0004_BF0C;
pub const STAGE44_BT_STEP_B_BOUNDARY: u32 = 0x0003_7158;
pub const STAGE44_BT_GLOBAL_PREDICATE_BOUNDARY: u32 = 0x0003_34F8;
pub const STAGE44_BT_GLOBAL_NOTIFY_BOUNDARY: u32 = 0x0003_BC94;
pub const STAGE44_BT_PRIMARY_MAP_BOUNDARY: u32 = 0x0002_E678;
pub const STAGE44_BT_FINAL_PREDICATE_BOUNDARY: u32 = 0x0005_8488;
pub const STAGE44_BT_ZERO_TAIL_BOUNDARY: u32 = 0x0005_833C;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage44PostGateState {
    /// Object byte +29.
    pub byte29: u8,
    /// Object halfword +236.
    pub word236: u16,
    /// Object dword +56.
    pub dword56: u32,
    /// Runtime word at shared-flags base +4 (0x208834).
    pub shared_flags_plus4: u32,
}

/// Opaque boundaries used by current 0x16D99C / legacy sub_16AAA0.
pub trait BtStage44PostGateBackend {
    /// Current 0x3CCDC(object, word236).
    fn probe(&mut self, word236: u16) -> u32;

    /// Current 0x3CC9E(object, probe_result). The binary forwards R1 from the probe result.
    fn probe_followup(&mut self, probe_result: u32);

    /// Current tail 0x3C3B0(object, 1).
    fn flagged_tail(&mut self, one: u32) -> u32;

    /// Current tail 0x2EC18(object).
    fn default_tail(&mut self) -> u32;
}

/// Safe source-level model of current 0x16D99C.
///
/// Exact locally visible behavior:
/// - when object byte +29 bit7 is set, call the probe with halfword +236;
/// - only probe result 1 invokes the follow-up, forwarding that result as argument 2;
/// - choose the 0x3C3B0 tail only when dword +56 bit3 is set, byte +29 bit7 is clear,
///   and runtime word 0x208834 bit11 is set;
/// - otherwise tail to 0x2EC18.
pub fn bt_stage44_post_gate<B: BtStage44PostGateBackend>(
    state: &BtStage44PostGateState,
    backend: &mut B,
) -> u32 {
    if state.byte29 & 0x80 != 0 {
        let probe_result = backend.probe(state.word236);
        if probe_result == 1 {
            backend.probe_followup(probe_result);
        }
    }

    if state.dword56 & 0x08 != 0
        && state.byte29 & 0x80 == 0
        && state.shared_flags_plus4 & 0x0800 != 0
    {
        backend.flagged_tail(1)
    } else {
        backend.default_tail()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage44PostSequenceState {
    /// Object dword +0 captured before any runtime calls.
    pub primary: u32,
    /// Object halfword +236.
    pub word236: u16,
    /// Object byte +29.
    pub byte29: u8,
    /// Object dword +52.
    pub dword52: u32,
    /// Object byte +28; bits 3..7 are rewritten to binary value 8 (0x40 in the byte).
    pub byte28: u8,
    /// Runtime dword at current 0x2090CC.
    pub global_2090cc: u32,
    /// Runtime dword at shared-flags base +8 (0x208838).
    pub shared_flags_plus8: u32,
}

/// Opaque boundaries used by current 0x16D9E8 / legacy sub_16AAEC.
pub trait BtStage44PostSequenceBackend {
    fn probe(&mut self, word236: u16) -> u32;
    fn probe_followup(&mut self, probe_result: u32);
    fn mode_write(&mut self, one: u32, zero: u32);

    /// Current 0x2EB58 is called with the object's pointer in R0 while R2 still contains
    /// `dword52 << 21`. The caller then forwards *post-call* R2 to 0x6E9E0 without
    /// recomputing it. This trait therefore exposes both the incoming ABI value and the
    /// caller-volatile R2 observed after the call.
    fn bit10_prepare_post_r2(&mut self, incoming_r2: u32) -> u32;

    fn forward_primary(&mut self, zero: u32, primary: u32, forwarded_r2: u32);
    fn step_a(&mut self);
    fn step_b(&mut self);
    fn global_predicate(&mut self) -> u32;
    fn global_notify(&mut self);
    fn map_primary(&mut self, primary: u32) -> u32;
    fn final_predicate(&mut self, mapped: u32) -> u32;
    fn zero_tail(&mut self, zero: u32) -> u32;
}

/// Safe source-level model of current 0x16D9E8.
///
/// Compiler stack-canary plumbing is intentionally omitted. All still-unresolved runtime
/// entries are traits. The caller-volatile R2 edge and the explicit zero tail argument are
/// modeled literally because both are observable binary behavior.
pub fn bt_stage44_post_sequence<B: BtStage44PostSequenceBackend>(
    state: &mut BtStage44PostSequenceState,
    backend: &mut B,
) -> u32 {
    let probe_result = backend.probe(state.word236);
    if probe_result == 1 && state.byte29 & 0x80 == 0 {
        backend.probe_followup(probe_result);
    }

    backend.mode_write(1, 0);

    let shifted_r2 = state.dword52.wrapping_shl(21);
    let forwarded_r2 = if state.dword52 & 0x0400 != 0 {
        backend.bit10_prepare_post_r2(shifted_r2)
    } else {
        shifted_r2
    };
    backend.forward_primary(0, state.primary, forwarded_r2);

    backend.step_a();
    backend.step_b();

    // BFI byte28, value 8, lsb 3, width 5.
    state.byte28 = (state.byte28 & 0x07) | 0x40;

    if state.global_2090cc == 1 && backend.global_predicate() == 1 {
        backend.global_notify();
    }

    let mapped = backend.map_primary(state.primary);
    if state.shared_flags_plus8 & 1 == 0 {
        return mapped;
    }

    let final_result = backend.final_predicate(mapped);
    if final_result == 0 {
        return 0;
    }

    // The current binary executes MOVS R0,#0 before the tail branch to 0x5833C.
    backend.zero_tail(0)
}

#[cfg(test)]
mod stage44_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct GateBackend {
        probe_result: u32,
        flagged_result: u32,
        default_result: u32,
        calls: Vec<(&'static str, u32)>,
    }

    impl BtStage44PostGateBackend for GateBackend {
        fn probe(&mut self, word236: u16) -> u32 {
            self.calls.push(("probe", word236 as u32));
            self.probe_result
        }
        fn probe_followup(&mut self, result: u32) {
            self.calls.push(("followup", result));
        }
        fn flagged_tail(&mut self, one: u32) -> u32 {
            self.calls.push(("flagged", one));
            self.flagged_result
        }
        fn default_tail(&mut self) -> u32 {
            self.calls.push(("default", 0));
            self.default_result
        }
    }

    #[test]
    fn post_gate_preserves_probe_and_tail_conditions() {
        let mut b = GateBackend {
            probe_result: 1,
            flagged_result: 0x33,
            default_result: 0x44,
            calls: Vec::new(),
        };
        let s = BtStage44PostGateState {
            byte29: 0x80,
            word236: 0x1234,
            dword56: 0x08,
            shared_flags_plus4: 0x0800,
        };
        assert_eq!(bt_stage44_post_gate(&s, &mut b), 0x44);
        assert_eq!(b.calls, [("probe", 0x1234), ("followup", 1), ("default", 0)]);

        let mut b = GateBackend {
            probe_result: 9,
            flagged_result: 0x55,
            default_result: 0x66,
            calls: Vec::new(),
        };
        let s = BtStage44PostGateState {
            byte29: 0,
            word236: 7,
            dword56: 0x08,
            shared_flags_plus4: 0x0800,
        };
        assert_eq!(bt_stage44_post_gate(&s, &mut b), 0x55);
        assert_eq!(b.calls, [("flagged", 1)]);
    }

    #[derive(Default)]
    struct SeqBackend {
        probe_result: u32,
        post_r2: u32,
        global_predicate_result: u32,
        mapped: u32,
        final_predicate_result: u32,
        zero_tail_result: u32,
        calls: Vec<(&'static str, u32, u32, u32)>,
    }

    impl BtStage44PostSequenceBackend for SeqBackend {
        fn probe(&mut self, w: u16) -> u32 {
            self.calls.push(("probe", w as u32, 0, 0));
            self.probe_result
        }
        fn probe_followup(&mut self, r: u32) {
            self.calls.push(("followup", r, 0, 0));
        }
        fn mode_write(&mut self, one: u32, zero: u32) {
            self.calls.push(("mode", one, zero, 0));
        }
        fn bit10_prepare_post_r2(&mut self, incoming: u32) -> u32 {
            self.calls.push(("prepare", incoming, 0, 0));
            self.post_r2
        }
        fn forward_primary(&mut self, z: u32, p: u32, r2: u32) {
            self.calls.push(("forward", z, p, r2));
        }
        fn step_a(&mut self) { self.calls.push(("step_a", 0, 0, 0)); }
        fn step_b(&mut self) { self.calls.push(("step_b", 0, 0, 0)); }
        fn global_predicate(&mut self) -> u32 {
            self.calls.push(("global_pred", 0, 0, 0));
            self.global_predicate_result
        }
        fn global_notify(&mut self) { self.calls.push(("notify", 0, 0, 0)); }
        fn map_primary(&mut self, p: u32) -> u32 {
            self.calls.push(("map", p, 0, 0));
            self.mapped
        }
        fn final_predicate(&mut self, m: u32) -> u32 {
            self.calls.push(("final_pred", m, 0, 0));
            self.final_predicate_result
        }
        fn zero_tail(&mut self, z: u32) -> u32 {
            self.calls.push(("zero_tail", z, 0, 0));
            self.zero_tail_result
        }
    }

    #[test]
    fn post_sequence_forwards_post_call_r2_not_the_pre_call_shift() {
        let mut b = SeqBackend {
            probe_result: 1,
            post_r2: 0xDEAD_BEEF,
            global_predicate_result: 1,
            mapped: 0x1234,
            final_predicate_result: 0,
            zero_tail_result: 0x9999,
            calls: Vec::new(),
        };
        let mut s = BtStage44PostSequenceState {
            primary: 0xCAFE,
            word236: 0x2222,
            byte29: 0,
            dword52: 0x0400,
            byte28: 0xA5,
            global_2090cc: 1,
            shared_flags_plus8: 1,
        };
        assert_eq!(bt_stage44_post_sequence(&mut s, &mut b), 0);
        assert_eq!(s.byte28, (0xA5 & 7) | 0x40);
        assert!(b.calls.contains(&("prepare", 0x8000_0000, 0, 0)));
        assert!(b.calls.contains(&("forward", 0, 0xCAFE, 0xDEAD_BEEF)));
        assert!(b.calls.contains(&("notify", 0, 0, 0)));
        assert!(!b.calls.iter().any(|x| x.0 == "zero_tail"));
    }

    #[test]
    fn post_sequence_explicitly_tails_with_zero_after_nonzero_final_predicate() {
        let mut b = SeqBackend {
            probe_result: 7,
            post_r2: 0,
            global_predicate_result: 0,
            mapped: 0x77,
            final_predicate_result: 5,
            zero_tail_result: 0xABCD,
            calls: Vec::new(),
        };
        let mut s = BtStage44PostSequenceState {
            primary: 0x99,
            word236: 3,
            byte29: 0x80,
            dword52: 2,
            byte28: 0xFF,
            global_2090cc: 0,
            shared_flags_plus8: 1,
        };
        assert_eq!(bt_stage44_post_sequence(&mut s, &mut b), 0xABCD);
        assert!(b.calls.contains(&("forward", 0, 0x99, 2u32 << 21)));
        assert!(b.calls.contains(&("zero_tail", 0, 0, 0)));
        assert!(!b.calls.iter().any(|x| x.0 == "followup"));
    }
}

/// Stage 45: current record-window maintenance at `0x16DB4C`.
///
/// The body is the sole whole-current-code relocation-normalized match of
/// legacy `sub_16AB80`. The only unresolved runtime call remains an opaque
/// trait boundary; the 116-byte zeroing path is modeled directly because
/// current `0x3D24` is already proven memset-like.
pub const STAGE45_CURRENT_BT_RECORD_MAINTENANCE_ADDR: u32 = 0x0016_DB4C;
pub const STAGE45_BT_PENDING15_BOUNDARY: u32 = 0x0003_B04A;
pub const STAGE45_BT_MEMSET_BOUNDARY: u32 = ROM_MEMSET_ADDR;
pub const STAGE45_BT_MATCH_MASK: u32 = 0x0003_0078;
pub const STAGE45_BT_MATCH_VALUE: u32 = 0x0003_0018;
pub const STAGE45_BT_WINDOW_BYTES: usize = 116;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage45ObjectState {
    /// Object dword +144. Its low byte is also used by the alternate gate.
    pub dword144: u32,
    /// Object byte +146.
    pub byte146: u8,
    /// Object byte +149.
    pub byte149: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtStage45RecordState {
    /// State byte +1.
    pub index: u8,
    /// State byte +14.
    pub clear_pending: u8,
    /// State byte +15.
    pub boundary_pending: u8,
    /// Exact state bytes +16..+131. In particular byte +20 is window[4]
    /// and byte +22 is window[6].
    pub window: [u8; STAGE45_BT_WINDOW_BYTES],
}

impl Default for BtStage45RecordState {
    fn default() -> Self {
        Self { index: 0, clear_pending: 0, boundary_pending: 0, window: [0; STAGE45_BT_WINDOW_BYTES] }
    }
}

impl BtStage45RecordState {
    pub const fn flags20(&self) -> u8 { self.window[4] }
    pub fn set_flags20(&mut self, value: u8) { self.window[4] = value; }
    pub const fn counter22(&self) -> u8 { self.window[6] }
    pub fn set_counter22(&mut self, value: u8) { self.window[6] = value; }
}

/// Still-opaque current `0x3B04A` boundary.
///
/// The current binary clears byte +15 before calling it, then re-reads byte
/// +14 afterwards, so this contract is allowed to mutate the state.
pub trait BtStage45Backend {
    fn pending15_boundary(
        &mut self,
        state_handle: u32,
        state: &mut BtStage45RecordState,
    ) -> u32;
}

/// Safe local-semantics model of current `0x16DB4C` / legacy `sub_16AB80`.
///
/// `object_handle` and `state_handle` retain the target's 32-bit pointer-shaped
/// return behavior without exposing host pointers. If the 116-byte clear path
/// runs, the observed memset-like return is exactly `state_handle + 16`.
pub fn bt_stage45_record_maintenance<B: BtStage45Backend>(
    object_handle: u32,
    object: &BtStage45ObjectState,
    state_handle: u32,
    state: &mut BtStage45RecordState,
    backend: &mut B,
) -> u32 {
    let mut result = object_handle;

    if object.byte149 == 2 {
        if object.dword144 & STAGE45_BT_MATCH_MASK == STAGE45_BT_MATCH_VALUE {
            state.set_counter22(state.counter22().wrapping_add(1));
            state.set_flags20(state.flags20() | 0x10);
        } else {
            let low144 = object.dword144 as u8;
            let class = (low144 >> 3) & 0x0f;
            let low2 = object.byte146 & 0x03;
            if class > 2 && (low2 == 1 || low2 == 2) && object.byte146 & 0x04 == 0 {
                state.set_flags20(state.flags20() | 0x04);
            }
        }
    }

    if state.boundary_pending != 0 {
        state.boundary_pending = 0;
        result = backend.pending15_boundary(state_handle, state);
    }

    // Re-read after the opaque boundary: it is permitted to change byte +14.
    if state.clear_pending != 0 {
        state.clear_pending = 0;
        state.window.fill(0);
        state.index = 0;
        return state_handle.wrapping_add(16);
    }

    let flags = state.flags20();
    if flags & 1 == 0 {
        let limit = flags >> 5;
        if state.index < limit {
            state.index = state.index.wrapping_add(1);
        }
    }

    result
}

#[cfg(test)]
mod stage45_tests {
    use super::*;

    #[derive(Default)]
    struct Backend { ret: u32, set_clear: bool, calls: u32 }
    impl BtStage45Backend for Backend {
        fn pending15_boundary(&mut self, _h: u32, state: &mut BtStage45RecordState) -> u32 {
            self.calls += 1;
            if self.set_clear { state.clear_pending = 1; }
            self.ret
        }
    }

    #[test]
    fn exact_mask_and_alternate_gate_preserve_byte_updates() {
        let mut b = Backend::default();
        let mut s = BtStage45RecordState::default();
        s.set_counter22(0xff);
        let o = BtStage45ObjectState { dword144: 0x0003_0018, byte146: 0, byte149: 2 };
        assert_eq!(bt_stage45_record_maintenance(0x1111, &o, 0x2000, &mut s, &mut b), 0x1111);
        assert_eq!(s.counter22(), 0);
        assert_eq!(s.flags20(), 0x10);

        let mut s = BtStage45RecordState::default();
        let o = BtStage45ObjectState { dword144: 0x18, byte146: 1, byte149: 2 };
        assert_eq!(bt_stage45_record_maintenance(7, &o, 0x3000, &mut s, &mut b), 7);
        assert_eq!(s.flags20(), 0x04);
    }

    #[test]
    fn pending_boundary_return_is_overridden_by_post_call_clear() {
        let mut b = Backend { ret: 0xdead_beef, set_clear: true, calls: 0 };
        let mut s = BtStage45RecordState::default();
        s.boundary_pending = 1;
        s.index = 5;
        s.window.fill(0xa5);
        let o = BtStage45ObjectState::default();
        assert_eq!(bt_stage45_record_maintenance(0x1111, &o, 0x8000, &mut s, &mut b), 0x8010);
        assert_eq!(b.calls, 1);
        assert_eq!(s.boundary_pending, 0);
        assert_eq!(s.clear_pending, 0);
        assert_eq!(s.index, 0);
        assert!(s.window.iter().all(|&x| x == 0));
    }

    #[test]
    fn callback_return_and_index_gate_preserve_exact_order() {
        let mut b = Backend { ret: 0x1234_5678, set_clear: false, calls: 0 };
        let mut s = BtStage45RecordState::default();
        s.boundary_pending = 1;
        s.index = 2;
        s.set_flags20(3 << 5);
        let o = BtStage45ObjectState::default();
        assert_eq!(bt_stage45_record_maintenance(9, &o, 0x4000, &mut s, &mut b), 0x1234_5678);
        assert_eq!(s.index, 3);

        s.index = 1;
        s.set_flags20((7 << 5) | 1);
        assert_eq!(bt_stage45_record_maintenance(0xaa, &o, 0x4000, &mut s, &mut b), 0xaa);
        assert_eq!(s.index, 1);

        assert_eq!(STAGE45_CURRENT_BT_RECORD_MAINTENANCE_ADDR, 0x16DB4C);
        assert_eq!(STAGE45_BT_MEMSET_BOUNDARY, 0x3D24);
        assert_eq!(STAGE45_BT_MATCH_MASK, 0x30078);
        assert_eq!(STAGE45_BT_MATCH_VALUE, 0x30018);
    }
}

/// Stage 46: current indexed-record/state transition at `0x16DD22`.
///
/// The body is the globally unique relocation-normalized current match of legacy
/// `sub_16AD56`. Runtime entries remain opaque trait boundaries. In particular,
/// current `0x3AF86` receives `(result_17e2c, state_ptr)` even though one legacy
/// Hex-Rays rendering omitted those arguments.
pub const STAGE46_CURRENT_BT_INDEXED_STATE_ADDR: u32 = 0x0016_DD22;
pub const STAGE46_BT_INITIAL_BOUNDARY: u32 = 0x0001_7E2C;
pub const STAGE46_BT_STATE_BOUNDARY: u32 = 0x0003_AF86;
pub const STAGE46_BT_OBJECT_PREDICATE_BOUNDARY: u32 = 0x0003_C7C2;
pub const STAGE46_BT_RECORD_STRIDE: usize = 25;
pub const STAGE46_BT_RECORD_DWORD_BASE_OFFSET: usize = 53;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage46ObjectState {
    /// Object byte +14, passed by value to current 0x17E2C.
    pub byte14: u8,
    /// Object byte +15, copied into state byte +18 on one path.
    pub byte15: u8,
    /// Object byte +152; bits 3..6 participate in the entry gate.
    pub byte152: u8,
    /// Object byte +154; low two bits participate in the entry gate.
    pub byte154: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage46State {
    /// State byte +1; used as an unchecked firmware record index.
    pub byte1: u8,
    /// State halfword +4.
    pub word4: u16,
    pub byte14: u8,
    pub byte15: u8,
    pub word16: u16,
    pub byte18: u8,
    pub byte19: u8,
    pub byte20: u8,
}

/// Exact byte offset used by the binary for the indexed dword store:
/// `state + 53 + 25 * state.byte1`.
///
/// No bound is imposed here because the firmware performs no observed bound check.
pub const fn bt_stage46_indexed_dword_offset(index: u8) -> usize {
    STAGE46_BT_RECORD_DWORD_BASE_OFFSET + STAGE46_BT_RECORD_STRIDE * index as usize
}

pub trait BtStage46Backend {
    /// Current `0x17E2C(object.byte14)`.
    fn initial(&mut self, object_byte14: u8) -> u32;

    /// Perform the binary's unchecked dword write relative to the state base.
    /// Implementations must not silently clamp `byte_offset` to a guessed record count.
    fn write_state_dword_unchecked(&mut self, byte_offset: usize, value: u32);

    /// Current `0x3AF86(first_result, state_ptr)`.
    /// `state` is mutable because the opaque runtime receives the real state pointer and
    /// later firmware reads `word4` and `byte20` after this call.
    fn state_boundary(&mut self, first_result: u32, state: &mut BtStage46State) -> u32;

    /// Current `0x3C7C2(object_ptr)`; the caller reduces its return to `(r0 == 0)`.
    fn object_predicate(&mut self, object: &mut BtStage46ObjectState) -> u32;
}

/// Safe source-level model of current `0x16DD22` / legacy `sub_16AD56`.
///
/// The unchecked indexed store is delegated to the backend instead of inventing a host-side
/// array length. All scalar updates and return-shape distinctions follow the Thumb body.
pub fn bt_stage46_indexed_state_update<B: BtStage46Backend>(
    object: &mut BtStage46ObjectState,
    second_argument: u32,
    state: &mut BtStage46State,
    backend: &mut B,
) -> u32 {
    let first_result = backend.initial(object.byte14);

    let gate_a = (object.byte152 >> 3) & 0x0F;
    let gate_b = object.byte154 & 0x03;
    if gate_a <= 2 || (gate_b != 1 && gate_b != 2) {
        return first_result;
    }

    backend.write_state_dword_unchecked(
        bt_stage46_indexed_dword_offset(state.byte1),
        first_result.wrapping_mul(2),
    );

    if second_argument != 0 {
        let second_result = backend.state_boundary(first_result, state);
        let result = if second_result != 0 {
            state.byte18 = object.byte15;
            let predicate = backend.object_predicate(object);
            state.word16 = state.word4;
            state.byte20 = state.byte20.wrapping_add(0x20);
            let boolean = u32::from(predicate == 0);
            state.byte19 = boolean as u8;
            state.byte15 = 1;
            boolean
        } else {
            0
        };
        state.byte14 = 1;
        result
    } else {
        let old = state.byte20;
        let high = ((old >> 5).wrapping_add(1)) & 7;
        let mut new = (old & 0x1F) | (high << 5);
        if high > 3 {
            new |= 1;
        }
        state.byte20 = new;
        first_result
    }
}

#[cfg(test)]
mod stage46_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    struct B {
        first: u32,
        second: u32,
        predicate: u32,
        mutate_word4: Option<u16>,
        mutate_byte20: Option<u8>,
        writes: Vec<(usize, u32)>,
        calls: Vec<(&'static str, u32)>,
    }

    impl BtStage46Backend for B {
        fn initial(&mut self, v: u8) -> u32 {
            self.calls.push(("initial", v as u32));
            self.first
        }
        fn write_state_dword_unchecked(&mut self, o: usize, v: u32) {
            self.writes.push((o, v));
        }
        fn state_boundary(&mut self, first: u32, state: &mut BtStage46State) -> u32 {
            self.calls.push(("state", first));
            if let Some(v) = self.mutate_word4 { state.word4 = v; }
            if let Some(v) = self.mutate_byte20 { state.byte20 = v; }
            self.second
        }
        fn object_predicate(&mut self, _object: &mut BtStage46ObjectState) -> u32 {
            self.calls.push(("predicate", 0));
            self.predicate
        }
    }

    fn backend(first: u32) -> B {
        B { first, second: 0, predicate: 0, mutate_word4: None, mutate_byte20: None, writes: Vec::new(), calls: Vec::new() }
    }
    fn object() -> BtStage46ObjectState {
        BtStage46ObjectState { byte14: 7, byte15: 0xA5, byte152: 0x18, byte154: 1 }
    }
    fn state() -> BtStage46State {
        BtStage46State { byte1: 3, word4: 0x1234, byte14: 0, byte15: 0, word16: 0, byte18: 0, byte19: 0, byte20: 0 }
    }

    #[test]
    fn gate_fail_preserves_first_return_and_performs_no_indexed_write() {
        let mut b = backend(0x1122_3344);
        let mut o = object(); o.byte152 = 0x10;
        let mut s = state();
        assert_eq!(bt_stage46_indexed_state_update(&mut o, 1, &mut s, &mut b), 0x1122_3344);
        assert!(b.writes.is_empty());
        assert_eq!(b.calls, [("initial", 7)]);
    }

    #[test]
    fn zero_second_argument_keeps_unchecked_index_and_rotates_high_three_bits() {
        let mut b = backend(0x8000_0001);
        let mut o = object();
        let mut s = state(); s.byte1 = 0xFF; s.byte20 = 0x7A;
        assert_eq!(bt_stage46_indexed_state_update(&mut o, 0, &mut s, &mut b), 0x8000_0001);
        assert_eq!(b.writes, [(53 + 25 * 0xFFusize, 2)]);
        assert_eq!(s.byte20, 0x9B); // high 3 bits: 3 -> 4; low 5 preserved, bit0 forced.
        assert_eq!(b.calls, [("initial", 7)]);
    }

    #[test]
    fn zero_state_boundary_sets_only_byte14_and_returns_zero() {
        let mut b = backend(9); b.second = 0;
        let mut o = object(); let mut s = state(); s.byte14 = 0x80;
        assert_eq!(bt_stage46_indexed_state_update(&mut o, 1, &mut s, &mut b), 0);
        assert_eq!(s.byte14, 1);
        assert_eq!(s.byte15, 0);
        assert_eq!(b.calls, [("initial", 7), ("state", 9)]);
    }

    #[test]
    fn nonzero_state_boundary_uses_post_call_state_and_zero_predicate_boolean() {
        let mut b = backend(11); b.second = 5; b.predicate = 0; b.mutate_word4 = Some(0xBEEF); b.mutate_byte20 = Some(0xF0);
        let mut o = object(); let mut s = state();
        assert_eq!(bt_stage46_indexed_state_update(&mut o, 3, &mut s, &mut b), 1);
        assert_eq!(s.byte18, 0xA5);
        assert_eq!(s.word16, 0xBEEF); // read after opaque state boundary.
        assert_eq!(s.byte20, 0x10);   // post-call 0xF0 + 0x20 wraps as u8.
        assert_eq!((s.byte19, s.byte15, s.byte14), (1, 1, 1));
        assert_eq!(b.calls, [("initial", 7), ("state", 11), ("predicate", 0)]);
    }

    #[test]
    fn nonzero_object_predicate_becomes_zero_return() {
        let mut b = backend(3); b.second = 1; b.predicate = 0x99;
        let mut o = object(); let mut s = state();
        assert_eq!(bt_stage46_indexed_state_update(&mut o, 1, &mut s, &mut b), 0);
        assert_eq!(s.byte19, 0);
        assert_eq!(STAGE46_CURRENT_BT_INDEXED_STATE_ADDR, 0x16DD22);
        assert_eq!(STAGE46_BT_STATE_BOUNDARY, 0x3AF86);
    }
}

/// Stage 47: current lookup/slot-transfer and packed-state update at `0x16DE9C`.
///
/// This is the globally unique relocation-normalized current match of legacy
/// `sub_16AED0`. The two runtime calls remain opaque traits.
pub const STAGE47_CURRENT_BT_SLOT_TRANSFER_ADDR: u32 = 0x0016_DE9C;
pub const STAGE47_BT_LOOKUP_BOUNDARY: u32 = 0x0001_EE18;
pub const STAGE47_BT_RELEASE_BOUNDARY: u32 = 0x000B_0460;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage47InputState {
    /// Input byte +9; bit 0 is forced on.
    pub byte9: u8,
    /// Input byte +20; passed by value to the lookup boundary.
    pub byte20: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage47State {
    /// State halfword +26. Bits 0..12 are partially replaced from the new handle header.
    pub word26: u16,
    /// State byte +29; set to 2 before packed-header updates.
    pub byte29: u8,
}

pub trait BtStage47Backend {
    /// Current `0x1EE18(input.byte20)`. The binary performs no local null check.
    fn lookup(&mut self, selector: u8) -> u32;
    /// Current `0xB0460(old_slot)`, invoked unconditionally. Its return value is the
    /// function's final return and must survive all subsequent local state updates.
    fn release(&mut self, old_slot: u32) -> u32;

    fn lookup_byte11(&mut self, lookup: u32) -> u8;
    fn lookup_replacement(&mut self, lookup: u32) -> u32;
    fn set_lookup_byte11(&mut self, lookup: u32, value: u8);
    fn set_lookup_replacement(&mut self, lookup: u32, value: u32);

    fn replacement_byte2(&mut self, replacement: u32) -> u8;
    fn replacement_word2(&mut self, replacement: u32) -> u16;
}

const fn replace_bits_3_through_12(old: u16, value: u16) -> u16 {
    (old & !0x1FF8) | ((value & 0x03FF) << 3)
}

/// Safe source-level model of current `0x16DE9C` / legacy `sub_16AED0`.
///
/// Handle validity is deliberately delegated to the backend: the firmware performs no
/// local null test after lookup before dereferencing the returned object/replacement.
pub fn bt_stage47_slot_transfer<B: BtStage47Backend>(
    input: &mut BtStage47InputState,
    slot: &mut u32,
    state: &mut BtStage47State,
    backend: &mut B,
) -> u32 {
    let lookup = backend.lookup(input.byte20);
    let release_result = backend.release(*slot);

    let replacement = backend.lookup_replacement(lookup);
    *slot = replacement;
    state.byte29 = 2;

    match backend.lookup_byte11(lookup) & 0xC0 {
        0x40 => {
            let replacement_now = backend.lookup_replacement(lookup);
            let packed = (backend.replacement_byte2(replacement_now) >> 3) as u16;
            state.word26 = replace_bits_3_through_12(state.word26, packed);
        }
        0x80 => {
            let replacement_now = backend.lookup_replacement(lookup);
            let packed = backend.replacement_word2(replacement_now) >> 3;
            state.word26 = replace_bits_3_through_12(state.word26, packed);
        }
        _ => {}
    }

    let replacement_now = backend.lookup_replacement(lookup);
    let byte2 = backend.replacement_byte2(replacement_now);
    state.word26 = (state.word26 & !0x0007) | u16::from(byte2 & 0x07);

    input.byte9 |= 1;
    let byte11 = backend.lookup_byte11(lookup);
    backend.set_lookup_byte11(lookup, (byte11 & 0xC0) | 0x3C);
    backend.set_lookup_replacement(lookup, 0);

    release_result
}

#[cfg(test)]
mod stage47_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    struct B {
        lookup: u32,
        release_result: u32,
        byte11: u8,
        replacement: u32,
        byte2: u8,
        word2: u16,
        calls: Vec<(&'static str, u32, u32)>,
    }
    impl BtStage47Backend for B {
        fn lookup(&mut self, s:u8)->u32{self.calls.push(("lookup",s as u32,0));self.lookup}
        fn release(&mut self,h:u32)->u32{self.calls.push(("release",h,0));self.release_result}
        fn lookup_byte11(&mut self,l:u32)->u8{self.calls.push(("byte11",l,0));self.byte11}
        fn lookup_replacement(&mut self,l:u32)->u32{self.calls.push(("replacement",l,0));self.replacement}
        fn set_lookup_byte11(&mut self,l:u32,v:u8){self.calls.push(("set_byte11",l,v as u32));self.byte11=v}
        fn set_lookup_replacement(&mut self,l:u32,v:u32){self.calls.push(("set_replacement",l,v));self.replacement=v}
        fn replacement_byte2(&mut self,h:u32)->u8{self.calls.push(("byte2",h,0));self.byte2}
        fn replacement_word2(&mut self,h:u32)->u16{self.calls.push(("word2",h,0));self.word2}
    }
    fn backend(mode:u8)->B{B{lookup:0x1000,release_result:0xDEAD_BEEF,byte11:mode|3,replacement:0x2000,byte2:0xAD,word2:0xB6AD,calls:Vec::new()}}

    #[test]
    fn mode40_packs_byte2_preserves_release_return_and_transfer_order(){
        let mut b=backend(0x40);let mut input=BtStage47InputState{byte9:0x80,byte20:7};let mut slot=0x3333;let mut state=BtStage47State{word26:0xE007,byte29:0};
        assert_eq!(bt_stage47_slot_transfer(&mut input,&mut slot,&mut state,&mut b),0xDEAD_BEEF);
        assert_eq!(slot,0x2000);assert_eq!(state.byte29,2);assert_eq!(input.byte9,0x81);
        let expected=replace_bits_3_through_12(0xE007,(0xADu16>>3)&0x3FF);
        assert_eq!(state.word26,(expected&!7)|5);assert_eq!(b.byte11,0x7C);assert_eq!(b.replacement,0);
        assert_eq!(&b.calls[..2],&[("lookup",7,0),("release",0x3333,0)]);
    }

    #[test]
    fn mode80_uses_halfword_then_low_three_bits_from_byte(){
        let mut b=backend(0x80);b.word2=0x7BAD;b.byte2=0xA2;let mut input=BtStage47InputState{byte9:0,byte20:1};let mut slot=9;let mut state=BtStage47State{word26:0xA005,byte29:9};
        let _=bt_stage47_slot_transfer(&mut input,&mut slot,&mut state,&mut b);
        let expected=replace_bits_3_through_12(0xA005,0x7BAD>>3);
        assert_eq!(state.word26,(expected&!7)|2);
    }

    #[test]
    fn other_mode_preserves_bits3_through12_but_still_replaces_low_three(){
        let mut b=backend(0x00);b.byte2=6;let mut input=BtStage47InputState{byte9:2,byte20:3};let mut slot=0;let mut state=BtStage47State{word26:0x5ABC,byte29:0};
        let r=bt_stage47_slot_transfer(&mut input,&mut slot,&mut state,&mut b);
        assert_eq!(r,0xDEAD_BEEF);assert_eq!(state.word26,(0x5ABC&!7)|6);assert_eq!(input.byte9,3);
        assert!(!b.calls.iter().any(|x|x.0=="word2"));
    }

    #[test]
    fn provenance_constants_are_current(){
        assert_eq!(STAGE47_CURRENT_BT_SLOT_TRANSFER_ADDR,0x16DE9C);
        assert_eq!(STAGE47_BT_LOOKUP_BOUNDARY,0x1EE18);
        assert_eq!(STAGE47_BT_RELEASE_BOUNDARY,0xB0460);
    }
}

/// Stage 48: current lookup mode router and three-record transfer at `0x16DF10`.
///
/// This is the globally unique relocation-normalized current match of legacy
/// `sub_16AF44`. The internal tail to Stage 47 relocates coherently from legacy
/// `sub_16AED0`; the remaining runtime calls stay opaque traits.
pub const STAGE48_CURRENT_BT_MODE_ROUTER_ADDR: u32 = 0x0016_DF10;
pub const STAGE48_BT_CONTEXT_BOUNDARY: u32 = 0x0003_35AC;
pub const STAGE48_BT_LOOKUP_BOUNDARY: u32 = 0x0001_EE18;
pub const STAGE48_BT_PREDICATE_BOUNDARY: u32 = 0x0001_F3BC;
pub const STAGE48_BT_FINALIZE_BOUNDARY: u32 = 0x0001_F3E0;
pub const STAGE48_BT_RELEASE_BOUNDARY: u32 = 0x000B_0460;
pub const STAGE48_BT_AMBIENT_WORD_ADDR: u32 = 0x0031_89DC;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage48InputState {
    /// Input byte +0; bits 3..6 feed local packed-record metadata.
    pub byte0: u8,
    /// Input halfword +2 copied into a selected record.
    pub word2: u16,
    /// Input byte +9, consumed only if the function delegates to Stage 47.
    pub byte9: u8,
    /// Input byte +20, passed to both initial runtime lookups.
    pub byte20: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage48Links {
    /// First pointer-shaped dword at argument 1 +0.
    pub slot0: u32,
    /// Second pointer-shaped dword at argument 1 +4.
    pub slot1: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage48State {
    /// State halfword +26, shared with the Stage-47 delegated path.
    pub word26: u16,
    /// State byte +28, part of the early special gate.
    pub byte28: u8,
    /// State byte +29, part of the early gate and shared with Stage 47.
    pub byte29: u8,
}

pub trait BtStage48Backend: BtStage47Backend {
    /// Current `0x335AC(input.byte20)`; its returned object is locally read at byte +167.
    fn context_boundary(&mut self, selector: u8) -> u32;
    fn context_byte167(&mut self, context: u32) -> u8;

    fn set_lookup_byte12(&mut self, lookup: u32, value: u8);

    /// Two separate reads are required because firmware reloads this ambient word between
    /// the two indirect byte stores.
    fn ambient_word_3189dc(&mut self) -> u32;
    /// Perform the firmware's byte store through a pointer-shaped dword.
    fn write_indirect_byte(&mut self, pointer: u32, value: u8);

    /// Current `0x1F3BC(lookup)`.
    fn lookup_predicate(&mut self, lookup: u32) -> u32;

    /// Record `index` is one of 0, 1, 2; each record is 12 bytes apart and its pointer
    /// field is at lookup + 20 + 12*index.
    fn lookup_record_dword20(&mut self, lookup: u32, index: u8) -> u32;
    fn set_lookup_record_word24(&mut self, lookup: u32, index: u8, value: u16);
    fn set_lookup_record_byte24(&mut self, lookup: u32, index: u8, value: u8);
    fn set_lookup_record_word26(&mut self, lookup: u32, index: u8, value: u16);
    fn set_lookup_record_dword20(&mut self, lookup: u32, index: u8, value: u32);

    /// Current tail boundary `0x1F3E0(lookup, context)`.
    fn finalize_boundary(&mut self, lookup: u32, context: u32) -> u32;
}

const fn stage48_mode(byte11: u8) -> u8 {
    (byte11 >> 2) & 0x0F
}

const fn stage48_replace_mode(byte11: u8, mode: u8) -> u8 {
    (byte11 & !0x3C) | ((mode & 0x0F) << 2)
}

const fn stage48_replace_class(byte11: u8, class: u8) -> u8 {
    (byte11 & !0xC0) | ((class & 0x03) << 6)
}

fn stage48_init_record<B: BtStage48Backend>(
    lookup: u32,
    index: u8,
    input: &BtStage48InputState,
    pointer: u32,
    backend: &mut B,
) {
    backend.set_lookup_record_word24(lookup, index, 0);
    let packed = ((input.byte0 >> 3) & 0x0F) << 3;
    backend.set_lookup_record_byte24(lookup, index, packed);
    backend.set_lookup_record_word26(lookup, index, input.word2);
    backend.set_lookup_record_dword20(lookup, index, pointer);
}

fn stage48_delegate_stage47<B: BtStage48Backend>(
    input: &mut BtStage48InputState,
    links: &mut BtStage48Links,
    state: &mut BtStage48State,
    backend: &mut B,
) -> u32 {
    let mut stage47_input = BtStage47InputState {
        byte9: input.byte9,
        byte20: input.byte20,
    };
    let mut stage47_state = BtStage47State {
        word26: state.word26,
        byte29: state.byte29,
    };
    let result = bt_stage47_slot_transfer(
        &mut stage47_input,
        &mut links.slot0,
        &mut stage47_state,
        backend,
    );
    input.byte9 = stage47_input.byte9;
    state.word26 = stage47_state.word26;
    state.byte29 = stage47_state.byte29;
    result
}

/// Safe source-level model of current `0x16DF10` / legacy `sub_16AF44`.
///
/// Pointer dereferences, ambient memory and unresolved runtime calls remain backend
/// operations. The local control flow, packed-field updates, repeated reads, record
/// selection and the direct tail delegation into reconstructed Stage 47 are preserved.
pub fn bt_stage48_mode_router<B: BtStage48Backend>(
    input: &mut BtStage48InputState,
    links: &mut BtStage48Links,
    state: &mut BtStage48State,
    backend: &mut B,
) -> u32 {
    let context = backend.context_boundary(input.byte20);
    let lookup = backend.lookup(input.byte20);

    if state.byte28 == 2 && (state.byte29 == 2 || state.byte29 == 4) {
        let byte11 = backend.lookup_byte11(lookup);
        let mode = stage48_mode(byte11);
        if mode <= 2 {
            backend.set_lookup_byte11(lookup, stage48_replace_mode(byte11, 3));
            return backend.finalize_boundary(lookup, context);
        }
        if mode == 3 {
            let predicate = backend.lookup_predicate(lookup);
            if predicate == 0 {
                return 0;
            }
            let byte11_after = backend.lookup_byte11(lookup);
            backend.set_lookup_byte11(lookup, byte11_after | 0x3C);
            return predicate;
        }
        return lookup;
    }

    backend.set_lookup_byte12(lookup, input.byte20);

    let class = if backend.context_byte167(context) & 0xE0 == 0x20 {
        if ((input.byte0 >> 3) & 0x0F) <= 9 { 1 } else { 2 }
    } else {
        match input.byte0 & 0x78 {
            0x18 | 0x48 => 1,
            _ => 2,
        }
    };
    let byte11 = backend.lookup_byte11(lookup);
    backend.set_lookup_byte11(lookup, stage48_replace_class(byte11, class));

    let ambient0 = backend.ambient_word_3189dc();
    backend.write_indirect_byte(links.slot1, ambient0 as u8);
    let ambient1 = backend.ambient_word_3189dc();
    backend.write_indirect_byte(links.slot0, (ambient1 >> 8) as u8);

    let mode = stage48_mode(backend.lookup_byte11(lookup));
    match mode {
        0 | 1 => {
            if backend.lookup_replacement(lookup) != 0 {
                return stage48_delegate_stage47(input, links, state, backend);
            }
            let next_mode = (mode + 1) & 0x0F;
            let byte11_now = backend.lookup_byte11(lookup);
            backend.set_lookup_byte11(lookup, stage48_replace_mode(byte11_now, next_mode));
            if backend.lookup_record_dword20(lookup, next_mode) != 0 {
                return backend.release(links.slot0);
            }
            stage48_init_record(lookup, next_mode, input, links.slot0, backend);
            backend.finalize_boundary(lookup, context)
        }
        2 => {
            if backend.lookup_replacement(lookup) != 0 {
                return stage48_delegate_stage47(input, links, state, backend);
            }
            let record1 = backend.lookup_record_dword20(lookup, 1);
            let record2 = backend.lookup_record_dword20(lookup, 2);
            if record1 == 0 {
                stage48_init_record(lookup, 1, input, links.slot0, backend);
                let byte11_now = backend.lookup_byte11(lookup);
                backend.set_lookup_byte11(lookup, stage48_replace_mode(byte11_now, 1));
                return backend.finalize_boundary(lookup, context);
            }
            if record2 == 0 {
                stage48_init_record(lookup, 2, input, links.slot0, backend);
                return backend.finalize_boundary(lookup, context);
            }
            backend.release(links.slot0)
        }
        3 => {
            if backend.lookup_predicate(lookup) == 0 {
                return backend.release(links.slot0);
            }
            stage48_init_record(lookup, 0, input, links.slot0, backend);
            let byte11_now = backend.lookup_byte11(lookup);
            backend.set_lookup_byte11(lookup, stage48_replace_mode(byte11_now, 0));
            backend.finalize_boundary(lookup, context)
        }
        15 => {
            let byte11_now = backend.lookup_byte11(lookup);
            backend.set_lookup_byte11(lookup, stage48_replace_mode(byte11_now, 0));
            stage48_init_record(lookup, 0, input, links.slot0, backend);
            backend.finalize_boundary(lookup, context)
        }
        _ => lookup,
    }
}

#[cfg(test)]
mod stage48_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Clone, Copy, Debug, Default)]
    struct Record { dword20: u32, word24: u16, word26: u16 }

    struct B {
        context: u32,
        lookup: u32,
        byte167: u8,
        byte11: u8,
        byte12: u8,
        replacement: u32,
        replacement_byte2: u8,
        replacement_word2: u16,
        release_result: u32,
        predicate: u32,
        predicate_mutate_byte11: Option<u8>,
        finalize_result: u32,
        ambient_reads: Vec<u32>,
        records: [Record; 3],
        indirect: Vec<(u32, u8)>,
        calls: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage47Backend for B {
        fn lookup(&mut self, selector: u8) -> u32 {
            self.calls.push(("lookup", selector as u32, 0)); self.lookup
        }
        fn release(&mut self, old_slot: u32) -> u32 {
            self.calls.push(("release", old_slot, 0)); self.release_result
        }
        fn lookup_byte11(&mut self, _lookup: u32) -> u8 { self.byte11 }
        fn lookup_replacement(&mut self, _lookup: u32) -> u32 { self.replacement }
        fn set_lookup_byte11(&mut self, _lookup: u32, value: u8) { self.byte11 = value; }
        fn set_lookup_replacement(&mut self, _lookup: u32, value: u32) { self.replacement = value; }
        fn replacement_byte2(&mut self, _replacement: u32) -> u8 { self.replacement_byte2 }
        fn replacement_word2(&mut self, _replacement: u32) -> u16 { self.replacement_word2 }
    }

    impl BtStage48Backend for B {
        fn context_boundary(&mut self, selector: u8) -> u32 {
            self.calls.push(("context", selector as u32, 0)); self.context
        }
        fn context_byte167(&mut self, _context: u32) -> u8 { self.byte167 }
        fn set_lookup_byte12(&mut self, _lookup: u32, value: u8) { self.byte12 = value; }
        fn ambient_word_3189dc(&mut self) -> u32 {
            let v = if self.ambient_reads.is_empty() { 0 } else { self.ambient_reads.remove(0) };
            self.calls.push(("ambient", v, 0)); v
        }
        fn write_indirect_byte(&mut self, pointer: u32, value: u8) {
            self.indirect.push((pointer, value));
        }
        fn lookup_predicate(&mut self, _lookup: u32) -> u32 {
            self.calls.push(("predicate", 0, 0));
            if let Some(v) = self.predicate_mutate_byte11 { self.byte11 = v; }
            self.predicate
        }
        fn lookup_record_dword20(&mut self, _lookup: u32, index: u8) -> u32 {
            self.records[index as usize].dword20
        }
        fn set_lookup_record_word24(&mut self, _lookup: u32, index: u8, value: u16) {
            self.records[index as usize].word24 = value;
        }
        fn set_lookup_record_byte24(&mut self, _lookup: u32, index: u8, value: u8) {
            self.records[index as usize].word24 = (self.records[index as usize].word24 & 0xFF00) | u16::from(value);
        }
        fn set_lookup_record_word26(&mut self, _lookup: u32, index: u8, value: u16) {
            self.records[index as usize].word26 = value;
        }
        fn set_lookup_record_dword20(&mut self, _lookup: u32, index: u8, value: u32) {
            self.records[index as usize].dword20 = value;
        }
        fn finalize_boundary(&mut self, lookup: u32, context: u32) -> u32 {
            self.calls.push(("finalize", lookup, context)); self.finalize_result
        }
    }

    fn backend(mode: u8) -> B {
        let mut ambient_reads = Vec::new();
        ambient_reads.push(0x1122_33D4);
        ambient_reads.push(0x5566_C300);
        B {
            context: 0x1111,
            lookup: 0x2222,
            byte167: 0x20,
            byte11: (mode & 0x0F) << 2,
            byte12: 0,
            replacement: 0,
            replacement_byte2: 0xAD,
            replacement_word2: 0x7BAD,
            release_result: 0xAABB_CCDD,
            predicate: 1,
            predicate_mutate_byte11: None,
            finalize_result: 0x5566_7788,
            ambient_reads,
            records: [Record::default(); 3],
            indirect: Vec::new(),
            calls: Vec::new(),
        }
    }
    fn input() -> BtStage48InputState {
        BtStage48InputState { byte0: 0x28, word2: 0xBEEF, byte9: 0x80, byte20: 7 }
    }
    fn links() -> BtStage48Links { BtStage48Links { slot0: 0x1000, slot1: 0x2000 } }
    fn state() -> BtStage48State { BtStage48State { word26: 0xE007, byte28: 0, byte29: 0 } }

    #[test]
    fn special_gate_modes_zero_through_two_promote_to_three_and_finalize() {
        let mut b = backend(1); let mut i = input(); let mut l = links();
        let mut s = state(); s.byte28 = 2; s.byte29 = 4;
        assert_eq!(bt_stage48_mode_router(&mut i, &mut l, &mut s, &mut b), 0x5566_7788);
        assert_eq!(stage48_mode(b.byte11), 3);
        assert!(b.indirect.is_empty());
    }

    #[test]
    fn special_gate_mode_three_rereads_byte11_after_predicate() {
        let mut b = backend(3); b.predicate = 0x77; b.predicate_mutate_byte11 = Some(0x81);
        let mut i = input(); let mut l = links(); let mut s = state(); s.byte28 = 2; s.byte29 = 2;
        assert_eq!(bt_stage48_mode_router(&mut i, &mut l, &mut s, &mut b), 0x77);
        assert_eq!(b.byte11, 0xBD);
    }

    #[test]
    fn mode_zero_with_existing_replacement_delegates_to_stage47() {
        let mut b = backend(0); b.replacement = 0x4444; b.replacement_byte2 = 0xAD;
        let mut i = input(); let mut l = links(); let mut s = state();
        let r = bt_stage48_mode_router(&mut i, &mut l, &mut s, &mut b);
        assert_eq!(r, 0xAABB_CCDD);
        assert_eq!(l.slot0, 0x4444);
        assert_eq!(i.byte9, 0x81);
        assert_eq!(s.byte29, 2);
        assert_eq!(b.byte11, 0x7C);
        assert_eq!(b.replacement, 0);
        assert_eq!(b.indirect, [(0x2000, 0xD4), (0x1000, 0xC3)]);
    }

    #[test]
    fn mode_two_prefers_empty_record_one_and_rewinds_mode_to_one() {
        let mut b = backend(2); let mut i = input(); let mut l = links(); let mut s = state();
        assert_eq!(bt_stage48_mode_router(&mut i, &mut l, &mut s, &mut b), 0x5566_7788);
        assert_eq!(b.records[1].dword20, 0x1000);
        assert_eq!(b.records[1].word24, 0x28);
        assert_eq!(b.records[1].word26, 0xBEEF);
        assert_eq!(stage48_mode(b.byte11), 1);
        assert_eq!(b.records[2].dword20, 0);
    }

    #[test]
    fn mode_two_uses_record_two_when_record_one_is_occupied() {
        let mut b = backend(2); b.records[1].dword20 = 9;
        let mut i = input(); let mut l = links(); let mut s = state();
        assert_eq!(bt_stage48_mode_router(&mut i, &mut l, &mut s, &mut b), 0x5566_7788);
        assert_eq!(b.records[2].dword20, 0x1000);
        assert_eq!(stage48_mode(b.byte11), 2);
    }

    #[test]
    fn mode_two_releases_current_slot_when_both_records_are_occupied() {
        let mut b = backend(2); b.records[1].dword20 = 1; b.records[2].dword20 = 2;
        let mut i = input(); let mut l = links(); let mut s = state();
        assert_eq!(bt_stage48_mode_router(&mut i, &mut l, &mut s, &mut b), 0xAABB_CCDD);
        assert!(b.calls.iter().any(|x|*x == ("release", 0x1000, 0)));
    }

    #[test]
    fn mode_three_predicate_success_initializes_record_zero_and_clears_mode() {
        let mut b = backend(3); let mut i = input(); let mut l = links(); let mut s = state();
        assert_eq!(bt_stage48_mode_router(&mut i, &mut l, &mut s, &mut b), 0x5566_7788);
        assert_eq!(b.records[0].dword20, 0x1000);
        assert_eq!(stage48_mode(b.byte11), 0);
    }

    #[test]
    fn mode_fifteen_clears_mode_initializes_record_zero_and_finalizes() {
        let mut b = backend(15); let mut i = input(); let mut l = links(); let mut s = state();
        assert_eq!(bt_stage48_mode_router(&mut i, &mut l, &mut s, &mut b), 0x5566_7788);
        assert_eq!(stage48_mode(b.byte11), 0);
        assert_eq!(b.records[0].dword20, 0x1000);
        assert_eq!(STAGE48_BT_AMBIENT_WORD_ADDR, 0x3189DC);
    }
}

/// Stage 49: current connection-state transition/commit routine at `0x16E0D8`.
///
/// This is the previously established relocation-normalized current match of legacy
/// `sub_16B10C`. Runtime calls and pointer-shaped payload/context accesses remain
/// opaque; this model preserves the locally visible control flow, packed fields,
/// global gates, scratch-cell aliasing, and state writes.
pub const STAGE49_CURRENT_BT_STATE_COMMIT_ADDR: u32 = 0x0016_E0D8;
pub const STAGE49_BT_ENTRY_BOUNDARY: u32 = 0x0003_A604;
pub const STAGE49_BT_BYTE14_BOUNDARY: u32 = 0x0001_7E2C;
pub const STAGE49_BT_CHAIN_BOUNDARY: u32 = 0x0001_7820;
pub const STAGE49_BT_CHAIN_PREDICATE_BOUNDARY: u32 = 0x0003_90E4;
pub const STAGE49_BT_EARLY_FINAL_BOUNDARY: u32 = 0x0002_02E8;
pub const STAGE49_BT_PRECHECK_BOUNDARY: u32 = 0x0002_5288;
pub const STAGE49_BT_NOTIFY_BOUNDARY: u32 = 0x0001_D104;
pub const STAGE49_BT_CAPABILITY_BOUNDARY: u32 = 0x0002_521C;
pub const STAGE49_BT_STATE_PREP_BOUNDARY: u32 = 0x0006_301C;
pub const STAGE49_BT_SUBSTATE_PREP_BOUNDARY: u32 = 0x0004_D57C;
pub const STAGE49_BT_STATE_GATE_BOUNDARY: u32 = 0x0002_1FC2;
pub const STAGE49_BT_SECONDARY_GATE_BOUNDARY: u32 = 0x0006_304C;
pub const STAGE49_BT_BYTE14_TEST_BOUNDARY: u32 = 0x0002_9778;
pub const STAGE49_BT_MODE_NOTIFY_BOUNDARY: u32 = 0x0004_D4DC;
pub const STAGE49_BT_BYTEA4_BOUNDARY: u32 = 0x0001_EFA8;
pub const STAGE49_BT_STATE_EARLY_RETURN_BOUNDARY: u32 = 0x0003_67CC;
pub const STAGE49_BT_OPTIONAL_STATE_BOUNDARY: u32 = 0x000A_F094;
pub const STAGE49_BT_STATE_POST_BOUNDARY: u32 = 0x0006_24E8;
pub const STAGE49_BT_SUBSTATE_COMMIT_BOUNDARY: u32 = 0x0002_53B0;
pub const STAGE49_BT_CONTEXT_BOUNDARY: u32 = 0x0003_35AC;
pub const STAGE49_BT_CONTEXT_CLASS_BOUNDARY: u32 = 0x0003_8918;
pub const STAGE49_BT_OBJECT_PREDICATE_BOUNDARY: u32 = 0x0003_C7C2;
pub const STAGE49_BT_FEATURE_PREDICATE_BOUNDARY: u32 = 0x0002_C79E;
pub const STAGE49_BT_CONTEXT_FEATURE_BOUNDARY: u32 = 0x0002_C78C;
pub const STAGE49_BT_LATE_STATE_BOUNDARY: u32 = 0x0006_29D0;
pub const STAGE49_BT_ALIAS_BOUNDARY: u32 = 0x0001_EBA0;
pub const STAGE49_BT_FINAL_PREP_BOUNDARY: u32 = 0x0006_329C;
pub const STAGE49_BT_OPTIONAL_FINAL_BOUNDARY: u32 = 0x0002_CAA8;
pub const STAGE49_BT_FINAL_BOUNDARY: u32 = 0x0003_A742;
pub const STAGE49_BT_EQUAL_ARG_BOUNDARY: u32 = 0x0002_5320;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage49State {
    pub word12: u16,
    pub byte14: u8,
    pub byte15: u8,
    pub byte5e: u8,
    pub dword68: u32,
    pub word78: u16,

    pub byte90: u8,
    pub byte94: u8,
    pub byte95: u8,
    pub byte97: u8,
    pub byte98: u8,
    pub byte99: u8,
    pub byte9a: u8,
    pub byte9b: u8,
    pub byte9c: u8,
    pub byte9e: u8,
    pub byte9f: u8,
    pub dworda0: u32,
    pub bytea4: u8,
    pub bytea5: u8,

    pub dwordf8: u32,
    pub word104: u16,
    pub byte114: u8,
    pub byte115: u8,
    pub byte116: u8,
    pub byte11b: u8,
    pub byte11e: u8,
    pub byte11f: u8,
    pub byte121: u8,
    pub byte124: u8,
    pub byte125: u8,
    pub byte129: u8,
    pub byte131: u8,
    pub byte134: u8,
    pub byte135: u8,
}

impl BtStage49State {
    pub const fn word98(&self) -> u16 {
        (self.byte98 as u16) | ((self.byte99 as u16) << 8)
    }
    pub const fn word9a(&self) -> u16 {
        (self.byte9a as u16) | ((self.byte9b as u16) << 8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BtStage49Globals {
    /// `0x206F78 + 4/+8/+10/+22`.
    pub g206f78_b4: u8,
    pub g206f78_b8: u8,
    pub g206f78_b10: u8,
    pub g206f78_b22: u8,

    /// `0x208338 + 19`.
    pub g208338_b19: u8,
    /// `0x209B98`.
    pub g209b98_b0: u8,

    /// `0x208B78` pointer-shaped global and its byte +59.
    pub g208b78_primary: u32,
    pub g208b7c_secondary: u32,
    pub g208bb3_b59: u8,

    pub g209b94_mask: u32,
    pub g2079b6_threshold: u16,

    /// The 16-byte table at `0x20289E`, indexed by `(state.byte98 >> 3) & 0x0F`.
    pub g20289e_mode_table: [u8; 16],
    pub g202854_threshold: u32,

    pub g215c20: u32,
    pub g202fa8_flags: u32,
    pub g208bb8_mask: u32,

    /// Global state snapshots at `0x318ACC` / `0x318AD0`.
    pub g318acc_snapshot: u32,
    pub g318ad0_snapshot: u32,

    /// Base index used with matrix base `0x208C9C`.
    pub g208c98_index: u32,

    pub g202868_b0: u8,
    pub g208b74_b0: u8,
    pub g202852_b0: u8,
    pub g208b6d_b0: u8,
    pub g202865_b0: u8,
    pub g202866_b0: u8,

    pub g207ba8_b0: u8,
    pub g207ba5_b0: u8,
    pub g208bbc_mask: u32,
    pub g20285b_b0: u8,
    pub g206fe0_b9: u8,
    pub g3186d0_flags: u32,
    pub g207fc1_b0: u8,
    pub g20b278_b0: u8,
}

impl Default for BtStage49Globals {
    fn default() -> Self {
        Self {
            g206f78_b4: 0,
            g206f78_b8: 0,
            g206f78_b10: 0,
            g206f78_b22: 0,
            g208338_b19: 0,
            g209b98_b0: 0,
            g208b78_primary: 0,
            g208b7c_secondary: 0,
            g208bb3_b59: 0,
            g209b94_mask: 0,
            g2079b6_threshold: 0,
            g20289e_mode_table: [0; 16],
            g202854_threshold: 0,
            g215c20: 0,
            g202fa8_flags: 0,
            g208bb8_mask: 0,
            g318acc_snapshot: 0,
            g318ad0_snapshot: 0,
            g208c98_index: 0,
            g202868_b0: 0,
            g208b74_b0: 0,
            g202852_b0: 0,
            g208b6d_b0: 0,
            g202865_b0: 0,
            g202866_b0: 0,
            g207ba8_b0: 0,
            g207ba5_b0: 0,
            g208bbc_mask: 0,
            g20285b_b0: 0,
            g206fe0_b9: 0,
            g3186d0_flags: 0,
            g207fc1_b0: 0,
            g20b278_b0: 0,
        }
    }
}

/// Opaque Stage-49 runtime surface.
///
/// `call*` methods model boundaries for which the current function does not pass a
/// state pointer. `state_pointer_call*` additionally expose the modeled state because
/// the binary passes either the state pointer or `state + 0x90`, so post-call reads must
/// observe possible mutations.
pub trait BtStage49Backend {
    fn call0(&mut self, addr: u32) -> u32;
    fn call1(&mut self, addr: u32, a0: u32) -> u32;
    fn call2(&mut self, addr: u32, a0: u32, a1: u32) -> u32;
    fn call3(&mut self, addr: u32, a0: u32, a1: u32, a2: u32) -> u32;

    fn state_pointer_call0(
        &mut self,
        addr: u32,
        pointer: u32,
        state: &mut BtStage49State,
    ) -> u32;

    fn state_pointer_call1(
        &mut self,
        addr: u32,
        pointer: u32,
        state: &mut BtStage49State,
        a1: u32,
    ) -> u32;

    fn context_word64(&mut self, context: u32) -> u16;
    fn context_byte167(&mut self, context: u32) -> u8;
    fn payload_byte0(&mut self, payload: u32) -> u8;
    fn payload_byte1(&mut self, payload: u32) -> u8;

    /// Firmware address is `0x208C9C + context_class*40 + global_index*20`.
    /// The local operation copies byte +18 into +19, then writes `value` to +18.
    fn shift_matrix_entry(&mut self, context_class: u32, global_index: u32, value: u8);

    /// Current `0x3A742(4, state, &scratch_arg1)`.
    fn final_boundary(
        &mut self,
        code: u32,
        state_addr: u32,
        state: &mut BtStage49State,
        scratch_arg1: &mut u32,
    ) -> u32;
}

const fn stage49_mode(byte98: u8) -> u8 {
    (byte98 >> 3) & 0x0F
}

const fn stage49_set_byte98_bit7(byte98: u8, value: bool) -> u8 {
    (byte98 & 0x7F) | if value { 0x80 } else { 0 }
}

const fn stage49_clear_local_fields(v: u32) -> u32 {
    v & !(0x0000_0078 | 0x0000_7C00)
}

const fn stage49_set_local_fields_one(v: u32) -> u32 {
    stage49_clear_local_fields(v) | 0x0000_0408
}

fn stage49_common_local_state<B: BtStage49Backend>(
    state_addr: u32,
    state: &mut BtStage49State,
    globals: &mut BtStage49Globals,
    backend: &mut B,
    scratch_state: &mut u32,
    scratch_arg1: &mut u32,
) {
    let role = state.byte15;

    if role == 1 {
        state.byte11b = 3;
        *scratch_arg1 = stage49_clear_local_fields(*scratch_arg1);

        let gate = backend.state_pointer_call0(
            STAGE49_BT_STATE_GATE_BOUNDARY,
            state_addr,
            state,
        );
        if gate == 0
            && globals.g208b78_primary == state_addr
            && (globals.g209b94_mask & state.dwordf8) == 0
            && state.byte94 == 2
            && stage49_mode(state.byte90) <= 1
        {
            state.byte124 = role;
            globals.g208b7c_secondary = state_addr;
        }
    } else {
        let use_cleared = if state.byte114 != 0
            && state.word104 > 3
            && state.byte11b > 1
            && state.byte11e == 0
        {
            let test = backend.call1(
                STAGE49_BT_BYTE14_TEST_BOUNDARY,
                u32::from(state.byte14),
            );
            test == 0 || state.word104 < globals.g2079b6_threshold
        } else {
            false
        };

        if use_cleared {
            state.byte11b = 3;
            *scratch_arg1 = stage49_clear_local_fields(*scratch_arg1);
        } else {
            state.byte11b = 1;
            *scratch_arg1 = stage49_set_local_fields_one(*scratch_arg1);
        }
    }

    let local_word = *scratch_arg1 as u16;
    *scratch_state = (*scratch_state & 0xFFFF_0000) | u32::from(local_word);
    globals.g318acc_snapshot = u32::from(local_word);
    state.byte9c = 1;

    if globals.g208338_b19 & 0x08 == 0 {
        let field = ((*scratch_arg1 as u8) >> 3) & 0x0F;
        let _ = backend.call3(
            STAGE49_BT_MODE_NOTIFY_BOUNDARY,
            1,
            u32::from(field),
            0,
        );
    }

    state.byte129 = 0;
    if globals.g206f78_b4 != 0 {
        let _ = backend.call2(
            STAGE49_BT_NOTIFY_BOUNDARY,
            1,
            u32::from(local_word >> 3),
        );
    }
}

fn stage49_tail<B: BtStage49Backend>(
    state_addr: u32,
    state: &mut BtStage49State,
    globals: &mut BtStage49Globals,
    backend: &mut B,
    scratch_state: &mut u32,
    scratch_arg1: &mut u32,
) -> u32 {
    if state.byte15 == 1
        && (state.dwordf8 & globals.g208bbc_mask) != 0
        && (state.byte99 & 1) != 0
    {
        state.byte131 = globals.g20285b_b0;
    }

    if globals.g208338_b19 & 0x10 != 0 {
        let _ = backend.state_pointer_call0(
            STAGE49_BT_LATE_STATE_BOUNDARY,
            state_addr,
            state,
        );
    }

    if globals.g206fe0_b9 != 0 {
        let _ = backend.call2(
            STAGE49_BT_ALIAS_BOUNDARY,
            u32::from(state.bytea5),
            *scratch_state,
        );
    }

    let _ = backend.state_pointer_call0(
        STAGE49_BT_FINAL_PREP_BOUNDARY,
        state_addr,
        state,
    );

    if state.byte5e != 0 {
        let packed = (state.word9a() & !0x0004) & 0x1FFF;
        if packed == 1 {
            globals.g3186d0_flags |= 1;
        } else {
            globals.g3186d0_flags &= !1;
        }
    }

    if globals.g207fc1_b0 != 0 && globals.g20b278_b0 != 0 {
        let _ = backend.state_pointer_call1(
            STAGE49_BT_OPTIONAL_FINAL_BOUNDARY,
            state_addr,
            state,
            1,
        );
    }

    backend.final_boundary(4, state_addr, state, scratch_arg1)
}

/// Safe source-level model of current `0x16E0D8` / legacy `sub_16B10C`.
///
/// `state_addr` is required because the firmware compares and stores the numeric state
/// pointer and also reuses the pushed `r0` stack cell: only its low halfword is replaced,
/// leaving the original address high half intact for current `0x1EBA0`.
pub fn bt_stage49_state_commit<B: BtStage49Backend>(
    state_addr: u32,
    state: &mut BtStage49State,
    arg1: u32,
    globals: &mut BtStage49Globals,
    backend: &mut B,
) -> u32 {
    // `push {r0-r8,lr}` leaves two scratch cells that matter semantically.
    let mut scratch_state = state_addr;
    let mut scratch_arg1 = arg1;

    if arg1 == 1 {
        state.byte97 = 0;
        state.byte115 = 0;
        state.byte116 = 0;
    }

    let entry = backend.state_pointer_call1(
        STAGE49_BT_ENTRY_BOUNDARY,
        state_addr,
        state,
        arg1,
    );
    if entry != 0 {
        let v0 = backend.call1(
            STAGE49_BT_BYTE14_BOUNDARY,
            u32::from(state.byte14),
        );
        let v1 = backend.call2(STAGE49_BT_CHAIN_BOUNDARY, v0, 1);
        let byte15_zero = u32::from(state.byte15 == 0);
        let chained = backend.call3(
            STAGE49_BT_CHAIN_PREDICATE_BOUNDARY,
            v1,
            u32::from(state.bytea4),
            byte15_zero,
        );
        if chained == 0 {
            return backend.call0(STAGE49_BT_EARLY_FINAL_BOUNDARY);
        }
    }

    let precheck = backend.call0(STAGE49_BT_PRECHECK_BOUNDARY);
    if precheck == 0 {
        if globals.g206f78_b22 != 0 {
            let _ = backend.call2(STAGE49_BT_NOTIFY_BOUNDARY, 0x32, 0x18);
        }
        return backend.call0(STAGE49_BT_EARLY_FINAL_BOUNDARY);
    }

    state.word12 = 0;
    if u32::from(state.byte15) == arg1 {
        state.byte94 = 0;
        state.byte95 = 0;
        let ret = backend.call0(STAGE49_BT_EQUAL_ARG_BOUNDARY);
        if state.byte5e != 0 {
            globals.g3186d0_flags &= !1;
        }
        if state.dword68 != 0 {
            state.word12 = state.word78;
        }
        return ret;
    }

    let capability = backend.call0(STAGE49_BT_CAPABILITY_BOUNDARY);
    let bit7 = if capability != 0 || globals.g208338_b19 & 0x08 != 0 {
        true
    } else {
        false
    };
    state.byte98 = stage49_set_byte98_bit7(state.byte98, bit7);

    if globals.g209b98_b0 >> 1 != 0 {
        state.byte98 = stage49_set_byte98_bit7(
            state.byte98,
            globals.g209b98_b0 & 1 != 0,
        );
    }

    let _ = backend.state_pointer_call0(
        STAGE49_BT_STATE_PREP_BOUNDARY,
        state_addr,
        state,
    );
    if globals.g208338_b19 & 0x10 == 0 {
        let _ = backend.state_pointer_call0(
            STAGE49_BT_SUBSTATE_PREP_BOUNDARY,
            state_addr.wrapping_add(0x90),
            state,
        );
    }

    // Saved r1 is reused as a local packed copy of state halfword +0x98.
    scratch_arg1 = (scratch_arg1 & 0xFFFF_0000) | u32::from(state.word98());

    let outer_common = if state.dworda0 == 0
        || state.byte90 & 0x80 == 0
        || state.byte135 != 0
    {
        true
    } else {
        let gate = backend.state_pointer_call0(
            STAGE49_BT_STATE_GATE_BOUNDARY,
            state_addr,
            state,
        );
        if gate == 0
            && state.byte15 == 1
            && globals.g208bb3_b59 == 0
            && globals.g208b78_primary != state_addr
            && state.byte9e == 0
        {
            true
        } else {
            backend.call0(STAGE49_BT_SECONDARY_GATE_BOUNDARY) != 0
        }
    };

    if outer_common {
        stage49_common_local_state(
            state_addr,
            state,
            globals,
            backend,
            &mut scratch_state,
            &mut scratch_arg1,
        );
        return stage49_tail(
            state_addr,
            state,
            globals,
            backend,
            &mut scratch_state,
            &mut scratch_arg1,
        );
    }

    // Alternate state-transition path at current 0x16E304.
    if state.byte15 == 0 {
        state.byte11b = 1;
    }

    if state.byte9f != 0 {
        state.byte9f = 0;
        state.byte99 ^= 0x02;
        let _ = backend.call1(
            STAGE49_BT_BYTEA4_BOUNDARY,
            u32::from(state.bytea4),
        );
    }

    if state.byte9e != 0 {
        state.byte134 = state.byte134.wrapping_add(1);
        state.byte129 = 2;

        let mode = stage49_mode(state.byte98);
        if u32::from(globals.g20289e_mode_table[mode as usize])
            > globals.g202854_threshold
        {
            let gate = backend.state_pointer_call0(
                STAGE49_BT_STATE_GATE_BOUNDARY,
                state_addr,
                state,
            );
            if gate == 0
                && !(0x18..=0x1A).contains(&state.bytea4)
            {
                let early = backend.state_pointer_call0(
                    STAGE49_BT_STATE_EARLY_RETURN_BOUNDARY,
                    state_addr,
                    state,
                );
                if early != 0 {
                    return early;
                }
            }
        }
    } else {
        state.byte134 = 0;
        state.byte129 = 1;
    }

    if globals.g215c20 != 0 {
        let _ = backend.state_pointer_call0(
            STAGE49_BT_OPTIONAL_STATE_BOUNDARY,
            state_addr,
            state,
        );
    }

    state.byte9e = 1;
    state.byte115 = 1;
    state.byte125 = 1;

    if globals.g202fa8_flags & (1 << 16) != 0 {
        globals.g208bb8_mask &= !state.dwordf8;
    }

    if globals.g208338_b19 & 0x10 != 0 {
        let _ = backend.state_pointer_call0(
            STAGE49_BT_STATE_POST_BOUNDARY,
            state_addr,
            state,
        );
    }

    // Current code reloads word +0x98 after the opaque calls and updates only the
    // low half of the pushed-r0 scratch cell.
    let current_word98 = state.word98();
    scratch_state = (scratch_state & 0xFFFF_0000) | u32::from(current_word98);
    globals.g318acc_snapshot = u32::from(current_word98);
    globals.g318ad0_snapshot = u32::from(state.word9a());

    let _ = backend.state_pointer_call0(
        STAGE49_BT_SUBSTATE_COMMIT_BOUNDARY,
        state_addr.wrapping_add(0x90),
        state,
    );

    let context = backend.call1(
        STAGE49_BT_CONTEXT_BOUNDARY,
        u32::from(state.bytea4),
    );
    if context != 0 {
        let context_word64 = backend.context_word64(context);
        let context_class = backend.call1(
            STAGE49_BT_CONTEXT_CLASS_BOUNDARY,
            u32::from(context_word64),
        );
        if context_class != 2 && (backend.context_byte167(context) >> 5) == 0 {
            let mode = stage49_mode(state.byte98);
            if (mode & 0x0B) == 0x0A || mode == 4 {
                backend.shift_matrix_entry(context_class, globals.g208c98_index, 1);
            } else if (mode & 0x0B) == 0x0B || mode == 8 {
                backend.shift_matrix_entry(context_class, globals.g208c98_index, 0);
            }
        }
    }

    let mode_now = stage49_mode(state.byte98);
    state.byte9c = globals.g20289e_mode_table[mode_now as usize];
    state.word12 = u16::from(state.byte9c).wrapping_sub(1);

    if state.byte15 == 0 {
        let gate = backend.state_pointer_call0(
            STAGE49_BT_STATE_GATE_BOUNDARY,
            state_addr,
            state,
        );
        if gate == 0 {
            if state.byte121 != 0 {
                let pred = backend.state_pointer_call0(
                    STAGE49_BT_OBJECT_PREDICATE_BOUNDARY,
                    state_addr,
                    state,
                );
                if pred != 0 {
                    state.byte11e = globals.g202868_b0;
                    state.byte11f = globals.g208b74_b0;
                } else {
                    state.byte11e = globals.g202852_b0;
                    state.byte11f = globals.g208b6d_b0;
                }
            } else {
                state.byte11e = globals.g202865_b0;
                state.byte11f = globals.g202866_b0;
            }
        }
    }

    if globals.g206f78_b4 != 0 {
        let packed = u32::from((state.word98() >> 3) & 0x007F)
            | (u32::from(state.word9a()) << 7);
        let _ = backend.call2(STAGE49_BT_NOTIFY_BOUNDARY, 3, packed);
        if state.byte129 != 2 {
            let _ = backend.call2(STAGE49_BT_NOTIFY_BOUNDARY, 0x65, 1);
        }
    }

    if globals.g206f78_b8 != 0
        && (state.byte9a & 3) == 3
        && state.byte129 != 2
    {
        let first = backend.payload_byte0(state.dworda0);
        let value = if first & 0xFE == 0xFE {
            u32::from(backend.payload_byte1(state.dworda0).wrapping_add(0x43))
        } else {
            u32::from(first >> 1)
        };
        let _ = backend.call2(STAGE49_BT_NOTIFY_BOUNDARY, 0x5D, value);
    }

    if globals.g206f78_b10 != 0 && (state.byte9a & 3) != 3 {
        let packed = u32::from(state.bytea4 & 0x0F)
            | (u32::from((state.word9a() >> 3) & 0x03FF) << 4);
        let _ = backend.call2(STAGE49_BT_NOTIFY_BOUNDARY, 0x0D, packed);
    }

    if globals.g207ba8_b0 != 0 {
        let mode = stage49_mode(state.byte98);
        let feature = backend.call2(
            STAGE49_BT_FEATURE_PREDICATE_BOUNDARY,
            u32::from(mode),
            u32::from(state.bytea5),
        );
        if feature != 0 && state.dworda0 != 0 {
            let context = backend.call1(
                STAGE49_BT_CONTEXT_BOUNDARY,
                u32::from(state.bytea4),
            );
            if context != 0 {
                let _ = backend.call1(
                    STAGE49_BT_CONTEXT_FEATURE_BOUNDARY,
                    context,
                );
            }
        }

        // Firmware re-reads mode and payload pointer after the opaque calls.
        if stage49_mode(state.byte98) > 3 && state.dworda0 != 0 {
            globals.g207ba5_b0 = 1;
        }
    }

    stage49_tail(
        state_addr,
        state,
        globals,
        backend,
        &mut scratch_state,
        &mut scratch_arg1,
    )
}

#[cfg(test)]
mod stage49_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        entry: u32,
        chain_predicate: u32,
        early_final: u32,
        precheck: u32,
        equal_ret: u32,
        capability: u32,
        state_gate: u32,
        secondary_gate: u32,
        early_state_ret: u32,
        final_ret: u32,
        context: u32,
        context_word64: u16,
        context_byte167: u8,
        context_class: u32,
        payload0: u8,
        payload1: u8,
        calls: Vec<(u32, u32, u32, u32)>,
        matrix: Vec<(u32, u32, u8)>,
        final_scratch: u32,
    }

    impl BtStage49Backend for B {
        fn call0(&mut self, addr:u32)->u32 {
            self.calls.push((addr,0,0,0));
            match addr {
                STAGE49_BT_EARLY_FINAL_BOUNDARY => self.early_final,
                STAGE49_BT_PRECHECK_BOUNDARY => self.precheck,
                STAGE49_BT_CAPABILITY_BOUNDARY => self.capability,
                STAGE49_BT_SECONDARY_GATE_BOUNDARY => self.secondary_gate,
                STAGE49_BT_EQUAL_ARG_BOUNDARY => self.equal_ret,
                _ => 0,
            }
        }
        fn call1(&mut self, addr:u32,a0:u32)->u32 {
            self.calls.push((addr,a0,0,0));
            match addr {
                STAGE49_BT_CONTEXT_BOUNDARY => self.context,
                STAGE49_BT_CONTEXT_CLASS_BOUNDARY => self.context_class,
                _ => 0,
            }
        }
        fn call2(&mut self, addr:u32,a0:u32,a1:u32)->u32 {
            self.calls.push((addr,a0,a1,0)); 0
        }
        fn call3(&mut self, addr:u32,a0:u32,a1:u32,a2:u32)->u32 {
            self.calls.push((addr,a0,a1,a2));
            if addr==STAGE49_BT_CHAIN_PREDICATE_BOUNDARY { self.chain_predicate } else { 0 }
        }
        fn state_pointer_call0(&mut self, addr:u32,p:u32,_s:&mut BtStage49State)->u32 {
            self.calls.push((addr,p,0,0));
            match addr {
                STAGE49_BT_STATE_GATE_BOUNDARY => self.state_gate,
                STAGE49_BT_STATE_EARLY_RETURN_BOUNDARY => self.early_state_ret,
                _ => 0,
            }
        }
        fn state_pointer_call1(&mut self, addr:u32,p:u32,_s:&mut BtStage49State,a1:u32)->u32 {
            self.calls.push((addr,p,a1,0));
            if addr==STAGE49_BT_ENTRY_BOUNDARY { self.entry } else { 0 }
        }
        fn context_word64(&mut self,_:u32)->u16 { self.context_word64 }
        fn context_byte167(&mut self,_:u32)->u8 { self.context_byte167 }
        fn payload_byte0(&mut self,_:u32)->u8 { self.payload0 }
        fn payload_byte1(&mut self,_:u32)->u8 { self.payload1 }
        fn shift_matrix_entry(&mut self,c:u32,g:u32,v:u8){self.matrix.push((c,g,v));}
        fn final_boundary(&mut self,_:u32,_:u32,_:&mut BtStage49State,s:&mut u32)->u32 {
            self.final_scratch=*s; self.final_ret
        }
    }

    #[test]
    fn entry_one_clears_three_bytes_and_zero_chain_takes_early_final() {
        let mut s=BtStage49State{byte97:9,byte115:8,byte116:7,byte14:2,byte15:0,bytea4:4,..Default::default()};
        let mut g=BtStage49Globals::default();
        let mut b=B{entry:1,chain_predicate:0,early_final:0xDEAD_BEEF,..Default::default()};
        assert_eq!(bt_stage49_state_commit(0x0020_1234,&mut s,1,&mut g,&mut b),0xDEAD_BEEF);
        assert_eq!((s.byte97,s.byte115,s.byte116),(0,0,0));
        assert_eq!(b.calls.last().unwrap().0,STAGE49_BT_EARLY_FINAL_BOUNDARY);
    }

    #[test]
    fn equal_argument_path_clears_state_and_preserves_equal_boundary_return() {
        let mut s=BtStage49State{byte15:1,byte94:2,byte95:3,byte5e:1,dword68:1,word78:0x3456,..Default::default()};
        let mut g=BtStage49Globals{g3186d0_flags:0x55,..Default::default()};
        let mut b=B{precheck:7,equal_ret:0xCAFE_BABE,..Default::default()};
        let r=bt_stage49_state_commit(0x0020_1000,&mut s,1,&mut g,&mut b);
        assert_eq!(r,0xCAFE_BABE);
        assert_eq!((s.byte94,s.byte95,s.word12),(0,0,0x3456));
        assert_eq!(g.g3186d0_flags & 1,0);
    }

    #[test]
    fn common_path_builds_exact_local_fields_and_stack_alias() {
        let mut s=BtStage49State{byte15:2,byte98:0,byte99:0,bytea4:9,..Default::default()};
        let mut g=BtStage49Globals{g208338_b19:0x08,..Default::default()};
        let mut b=B{precheck:1,capability:0,final_ret:0x1234_5678,..Default::default()};
        let r=bt_stage49_state_commit(0x0021_9ABC,&mut s,3,&mut g,&mut b);
        assert_eq!(r,0x1234_5678);
        assert_eq!(g.g318acc_snapshot,0x0488);
        assert_eq!(b.final_scratch,0x0000_0488);
        assert_eq!(s.byte11b,1);
        assert_eq!(s.byte9c,1);
    }

    #[test]
    fn outer_gate_primary_mismatch_and_zero_byte9e_bypasses_secondary_gate() {
        let mut s=BtStage49State{
            byte15:1,byte90:0x80,byte9e:0,dworda0:1,byte98:0,
            ..Default::default()
        };
        let mut g=BtStage49Globals{g208b78_primary:0x0020_9999,..Default::default()};
        let mut b=B{precheck:1,state_gate:0,secondary_gate:0,final_ret:0x44,..Default::default()};
        let r=bt_stage49_state_commit(0x0020_0100,&mut s,3,&mut g,&mut b);
        assert_eq!(r,0x44);
        assert!(!b.calls.iter().any(|x|x.0==STAGE49_BT_SECONDARY_GATE_BOUNDARY));
        assert_eq!(s.byte11b,3);
    }

    #[test]
    fn outer_gate_primary_match_reaches_secondary_gate() {
        let state_addr=0x0020_0100;
        let mut s=BtStage49State{
            byte15:1,byte90:0x80,byte9e:0,dworda0:1,byte98:0,
            ..Default::default()
        };
        let mut g=BtStage49Globals{g208b78_primary:state_addr,..Default::default()};
        let mut b=B{precheck:1,state_gate:0,secondary_gate:1,final_ret:0x45,..Default::default()};
        let r=bt_stage49_state_commit(state_addr,&mut s,3,&mut g,&mut b);
        assert_eq!(r,0x45);
        assert!(b.calls.iter().any(|x|x.0==STAGE49_BT_SECONDARY_GATE_BOUNDARY));
    }

    #[test]
    fn alternate_path_zero_previous_state_sets_one_and_commits_flags() {
        let mut s=BtStage49State{
            byte15:2,byte90:0x80,byte9e:0,byte134:9,dworda0:1,byte98:0x18,
            ..Default::default()
        };
        let mut g=BtStage49Globals::default();
        let mut b=B{precheck:1,state_gate:1,secondary_gate:0,final_ret:7,..Default::default()};
        let r=bt_stage49_state_commit(0x0020_0100,&mut s,3,&mut g,&mut b);
        assert_eq!(r,7);
        assert_eq!((s.byte134,s.byte129,s.byte9e,s.byte115,s.byte125),(0,1,1,1,1));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE49_CURRENT_BT_STATE_COMMIT_ADDR,0x16E0D8);
        assert_eq!(STAGE49_BT_ENTRY_BOUNDARY,0x3A604);
        assert_eq!(STAGE49_BT_CONTEXT_BOUNDARY,0x335AC);
        assert_eq!(STAGE49_BT_FINAL_BOUNDARY,0x3A742);
    }
}

/// Stage 50: current lookup-slot maintenance routine at `0x16DDAC`.
///
/// This is the previously established relocation-normalized current match of legacy
/// `sub_16ADE0`. Current `0x1F270` and `0x780` remain opaque boundaries; the model
/// preserves only their observed arguments, return flow, slot rereads, and write order.
pub const STAGE50_CURRENT_BT_SLOT_MAINTENANCE_ADDR: u32 = 0x0016_DDAC;
pub const STAGE50_BT_LOOKUP_BOUNDARY: u32 = 0x0001_EE18;
pub const STAGE50_BT_SLOT_TEST_BOUNDARY: u32 = 0x0001_F270;
pub const STAGE50_BT_GUARD_BOUNDARY: u32 = 0x0000_0780;
pub const STAGE50_BT_RELEASE_BOUNDARY: u32 = 0x000B_0460;
pub const STAGE50_BT_AMBIENT_FLAGS_ADDR: u32 = 0x0020_8338;

const STAGE50_SLOT_10: u8 = 0x10;
const STAGE50_SLOT_14: u8 = 0x14;
const STAGE50_SLOT_20: u8 = 0x20;
const STAGE50_SLOT_2C: u8 = 0x2C;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage50Input {
    /// Safe flattening of the firmware's `*(*(arg0 + 4))` lookup selector load.
    pub lookup_selector: u32,
}

/// Opaque current-runtime surface used by Stage 50.
///
/// Lookup-object dwords are addressed by their exact firmware offsets so backend
/// implementations can preserve mutations performed by opaque calls between rereads.
pub trait BtStage50Backend {
    fn lookup(&mut self, selector: u32) -> u32;
    fn lookup_byte11(&mut self, lookup: u32) -> u8;
    fn lookup_dword(&mut self, lookup: u32, offset: u8) -> u32;
    fn set_lookup_dword(&mut self, lookup: u32, offset: u8, value: u32);
    fn object_byte2(&mut self, object: u32) -> u8;

    /// Current `0x1F270(lookup)`.
    fn slot_test(&mut self, lookup: u32) -> u32;
    /// Current `0x780(value)`; the returned token/value is passed back exactly where
    /// the binary does so without assigning a semantic name to the boundary.
    fn guard(&mut self, value: u32) -> u32;
    /// Current `0xB0460(handle)`. Return value is ignored by this routine.
    fn release(&mut self, handle: u32) -> u32;

    /// Byte at current ambient address `0x208338 + 0x13`, read only on the observed gate.
    fn ambient_flags_byte19(&mut self) -> u8;
}

const fn stage50_mode(byte11: u8) -> u8 {
    (byte11 >> 2) & 0x0F
}

fn stage50_release_if_nonzero<B: BtStage50Backend>(
    backend: &mut B,
    lookup: u32,
    offset: u8,
) {
    let handle = backend.lookup_dword(lookup, offset);
    if handle != 0 {
        let _ = backend.release(handle);
        backend.set_lookup_dword(lookup, offset, 0);
    }
}

/// Safe source-level model of current `0x16DDAC` / legacy `sub_16ADE0`.
///
/// The binary obtains the selector indirectly through the caller's pointer chain;
/// `BtStage50Input` exposes the already-loaded scalar. Handle validity and pointed-object
/// byte access remain backend responsibilities because firmware performs no local safety
/// validation beyond the explicit null checks represented here.
pub fn bt_stage50_lookup_slot_maintenance<B: BtStage50Backend>(
    input: &BtStage50Input,
    backend: &mut B,
) -> u32 {
    let lookup = backend.lookup(input.lookup_selector);
    let mut return_value = lookup;

    // An occupied primary slot exits immediately with the lookup handle.
    if backend.lookup_dword(lookup, STAGE50_SLOT_10) != 0 {
        return return_value;
    }

    let mode = stage50_mode(backend.lookup_byte11(lookup));
    let mut slot_test_result = 0u32;

    match mode {
        // TBB mode 0 selects dword +0x20.
        0 => {
            if backend.lookup_dword(lookup, STAGE50_SLOT_20) != 0 {
                return_value = backend.slot_test(lookup);
                slot_test_result = return_value;
            }
        }
        // TBB mode 1 selects dword +0x14.
        1 => {
            if backend.lookup_dword(lookup, STAGE50_SLOT_14) != 0 {
                return_value = backend.slot_test(lookup);
                slot_test_result = return_value;
            }
        }
        // TBB mode 2 selects dword +0x2C. A non-one test result performs two
        // unconditional release calls while bracketed by the opaque 0x780 boundary.
        2 => {
            if backend.lookup_dword(lookup, STAGE50_SLOT_2C) != 0 {
                slot_test_result = backend.slot_test(lookup);
                let token = backend.guard(1);
                if slot_test_result != 1 {
                    let slot20 = backend.lookup_dword(lookup, STAGE50_SLOT_20);
                    let _ = backend.release(slot20);
                    backend.set_lookup_dword(lookup, STAGE50_SLOT_20, 0);

                    let slot2c = backend.lookup_dword(lookup, STAGE50_SLOT_2C);
                    let _ = backend.release(slot2c);
                    backend.set_lookup_dword(lookup, STAGE50_SLOT_2C, 0);
                }
                return_value = backend.guard(token);
            }
        }
        // TBB mode 3 clears the three secondary slots plus +0x10, releasing only
        // nonzero handles. The guard-return is the live return value afterward.
        3 => {
            let token = backend.guard(1);
            stage50_release_if_nonzero(backend, lookup, STAGE50_SLOT_14);
            stage50_release_if_nonzero(backend, lookup, STAGE50_SLOT_20);
            stage50_release_if_nonzero(backend, lookup, STAGE50_SLOT_2C);
            stage50_release_if_nonzero(backend, lookup, STAGE50_SLOT_10);
            return_value = backend.guard(token);
        }
        // Modes 4..15 skip the TBB bodies.
        _ => {}
    }

    // The +0x14 pointed object's low two byte2 bits can bypass the ambient flag gate.
    let slot14 = backend.lookup_dword(lookup, STAGE50_SLOT_14);
    if slot14 != 0
        && backend.object_byte2(slot14) & 0x03 == 0
        && backend.ambient_flags_byte19() & 0x08 == 0
    {
        return return_value;
    }

    if slot_test_result != 1 {
        return return_value;
    }

    // Promotion path. Preserve firmware ordering: read +0x20 first, then +0x14,
    // publish +0x14 into +0x10 and clear it before conditionally releasing +0x20;
    // +0x2C is reread only after that release.
    let token = backend.guard(1);
    let old_slot20 = backend.lookup_dword(lookup, STAGE50_SLOT_20);
    let promoted = backend.lookup_dword(lookup, STAGE50_SLOT_14);
    backend.set_lookup_dword(lookup, STAGE50_SLOT_10, promoted);
    backend.set_lookup_dword(lookup, STAGE50_SLOT_14, 0);

    if old_slot20 != 0 {
        let _ = backend.release(old_slot20);
        backend.set_lookup_dword(lookup, STAGE50_SLOT_20, 0);
    }

    let old_slot2c = backend.lookup_dword(lookup, STAGE50_SLOT_2C);
    if old_slot2c != 0 {
        let _ = backend.release(old_slot2c);
        backend.set_lookup_dword(lookup, STAGE50_SLOT_2C, 0);
    }

    backend.guard(token)
}

#[cfg(test)]
mod stage50_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        lookup: u32,
        byte11: u8,
        slots: [u32; 4], // +10,+14,+20,+2C
        byte2: u8,
        test_result: u32,
        guard_first: u32,
        guard_second: u32,
        guard_calls: usize,
        ambient19: u8,
        calls: Vec<(&'static str, u32, u32)>,
    }

    impl B {
        fn index(offset:u8)->usize { match offset { 0x10=>0,0x14=>1,0x20=>2,0x2c=>3,_=>panic!("bad offset") } }
    }

    impl BtStage50Backend for B {
        fn lookup(&mut self,s:u32)->u32 { self.calls.push(("lookup",s,0)); self.lookup }
        fn lookup_byte11(&mut self,l:u32)->u8 { self.calls.push(("byte11",l,0)); self.byte11 }
        fn lookup_dword(&mut self,l:u32,o:u8)->u32 { self.calls.push(("read",l,o as u32)); self.slots[Self::index(o)] }
        fn set_lookup_dword(&mut self,l:u32,o:u8,v:u32) { self.calls.push(("write",o as u32,v)); self.slots[Self::index(o)]=v; let _=l; }
        fn object_byte2(&mut self,o:u32)->u8 { self.calls.push(("byte2",o,0)); self.byte2 }
        fn slot_test(&mut self,l:u32)->u32 { self.calls.push(("test",l,0)); self.test_result }
        fn guard(&mut self,v:u32)->u32 { self.calls.push(("guard",v,0)); let r=if self.guard_calls==0 {self.guard_first}else{self.guard_second}; self.guard_calls+=1; r }
        fn release(&mut self,h:u32)->u32 { self.calls.push(("release",h,0)); 0xDEAD_BEEF }
        fn ambient_flags_byte19(&mut self)->u8 { self.calls.push(("ambient",0,0)); self.ambient19 }
    }

    fn backend(mode:u8)->B {
        B { lookup:0x1000, byte11:(mode&0x0f)<<2, guard_first:0xA5A5, guard_second:0x5A5A, ..Default::default() }
    }

    #[test]
    fn occupied_primary_returns_lookup_without_mode_dispatch() {
        let mut b=backend(3); b.slots[0]=0x1111;
        let r=bt_stage50_lookup_slot_maintenance(&BtStage50Input{lookup_selector:7},&mut b);
        assert_eq!(r,0x1000);
        assert!(!b.calls.iter().any(|x|x.0=="byte11"));
    }

    #[test]
    fn mode_two_non_one_releases_both_slots_unconditionally_and_returns_guard_result() {
        let mut b=backend(2); b.slots[3]=0x3333; b.slots[2]=0; b.test_result=9;
        let r=bt_stage50_lookup_slot_maintenance(&BtStage50Input{lookup_selector:1},&mut b);
        assert_eq!(r,0x5A5A);
        let releases:Vec<u32>=b.calls.iter().filter(|x|x.0=="release").map(|x|x.1).collect();
        assert_eq!(releases,[0,0x3333]);
        assert_eq!((b.slots[2],b.slots[3]),(0,0));
        assert_eq!(b.guard_calls,2);
    }

    #[test]
    fn mode_three_releases_only_nonzero_handles_and_clears_all_four_slots() {
        let mut b=backend(3); b.slots=[0,0x14,0,0x2c];
        let r=bt_stage50_lookup_slot_maintenance(&BtStage50Input{lookup_selector:2},&mut b);
        assert_eq!(r,0x5A5A);
        let releases:Vec<u32>=b.calls.iter().filter(|x|x.0=="release").map(|x|x.1).collect();
        assert_eq!(releases,[0x14,0x2c]);
        assert_eq!(b.slots,[0,0,0,0]);
    }

    #[test]
    fn mode_one_test_one_promotes_slot14_then_releases_later_slots_in_order() {
        let mut b=backend(1);
        b.slots=[0,0x1414,0x2020,0x2c2c];
        b.byte2=1; // low two bits bypass ambient gating.
        b.test_result=1;
        let r=bt_stage50_lookup_slot_maintenance(&BtStage50Input{lookup_selector:3},&mut b);
        assert_eq!(r,0x5A5A);
        assert_eq!(b.slots,[0x1414,0,0,0]);
        let releases:Vec<u32>=b.calls.iter().filter(|x|x.0=="release").map(|x|x.1).collect();
        assert_eq!(releases,[0x2020,0x2c2c]);
        let write10=b.calls.iter().position(|x|*x==("write",0x10,0x1414)).unwrap();
        let release20=b.calls.iter().position(|x|*x==("release",0x2020,0)).unwrap();
        assert!(write10<release20);
    }

    #[test]
    fn zero_low_bits_require_ambient_bit3_for_promotion() {
        let mut b=backend(1); b.slots[1]=0x1414; b.test_result=1; b.byte2=0; b.ambient19=0;
        let r=bt_stage50_lookup_slot_maintenance(&BtStage50Input{lookup_selector:4},&mut b);
        assert_eq!(r,1); // live return remains 0x1F270 result.
        assert_eq!(b.slots[0],0);
        b.ambient19=0x08; b.guard_calls=0; b.calls.clear(); b.slots[1]=0x1414;
        let r2=bt_stage50_lookup_slot_maintenance(&BtStage50Input{lookup_selector:4},&mut b);
        assert_eq!(r2,0x5A5A);
        assert_eq!(b.slots[0],0x1414);
    }

    #[test]
    fn modes_above_three_return_lookup_when_primary_is_empty() {
        let mut b=backend(7);
        assert_eq!(bt_stage50_lookup_slot_maintenance(&BtStage50Input{lookup_selector:5},&mut b),0x1000);
        assert_eq!(b.guard_calls,0);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE50_CURRENT_BT_SLOT_MAINTENANCE_ADDR,0x16DDAC);
        assert_eq!(STAGE50_BT_LOOKUP_BOUNDARY,0x1EE18);
        assert_eq!(STAGE50_BT_SLOT_TEST_BOUNDARY,0x1F270);
        assert_eq!(STAGE50_BT_GUARD_BOUNDARY,0x780);
        assert_eq!(STAGE50_BT_RELEASE_BOUNDARY,0xB0460);
        assert_eq!(STAGE50_BT_AMBIENT_FLAGS_ADDR,0x208338);
    }
}

/// Stage 51: current record/state coordinator at `0x16DBDC`.
///
/// This is the previously established relocation-normalized current match of legacy
/// `sub_16AC10`. Runtime calls remain opaque boundaries; the model preserves the
/// exact local gates, unchecked record indexing, wrapping counters, post-call rereads,
/// and reset ordering visible in the current firmware.
pub const STAGE51_CURRENT_BT_RECORD_COORDINATOR_ADDR: u32 = 0x0016_DBDC;
pub const STAGE51_BT_CONTEXT_BOUNDARY: u32 = 0x0003_35AC;
pub const STAGE51_BT_SAMPLE_BOUNDARY: u32 = 0x0003_A6CC;
pub const STAGE51_BT_STATS_BOUNDARY: u32 = 0x0003_B04A;
pub const STAGE51_BT_WINDOW_BOUNDARY: u32 = 0x0000_3D24;

const STAGE51_RECORD_BYTE21: u8 = 0x21;
const STAGE51_RECORD_BYTE22: u8 = 0x22;
const STAGE51_RECORD_BYTE23: u8 = 0x23;
const STAGE51_RECORD_WORD26: u8 = 0x26;
const STAGE51_RECORD_BYTE28: u8 = 0x28;
const STAGE51_RECORD_BYTE29: u8 = 0x29;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage51ObjectState {
    pub byte15: u8,
    pub byte90: u8,
    pub byte91: u8,
    pub byte94: u8,
    pub byte96: u8,
    pub bytea4: u8,
    /// Current signed load at +0x113 is ultimately truncated back to one byte.
    pub byte113: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage51Stats {
    pub byte0: u8,
    /// Firmware uses this byte as an unchecked index with a stride of 25.
    pub byte1: u8,
    pub byte14: u8,
    pub byte15: u8,
    pub byte20: u8,
    pub byte22: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage51Event {
    pub byte0: u8,
    pub byte2: u8,
    pub byte4: u8,
    pub byte6: u8,
}

/// Opaque current-runtime and unchecked record-access surface used by Stage 51.
///
/// Record offsets are the exact offsets from `stats + 25 * stats.byte1` used by the
/// firmware. The backend owns bounds/aliasing behavior because the binary performs no
/// local bounds check on `stats.byte1`.
pub trait BtStage51Backend {
    /// Current `0x335AC(object.byteA4)`. Only the zero/nonzero result is consumed.
    fn context_boundary(&mut self, selector: u8) -> u32;
    /// Current zero-argument `0x3A6CC()`; only its low byte is stored locally.
    fn sample_boundary(&mut self) -> u32;

    fn record_byte(&mut self, index: u8, offset: u8) -> u8;
    fn set_record_byte(&mut self, index: u8, offset: u8, value: u8);
    fn record_word(&mut self, index: u8, offset: u8) -> u16;
    fn set_record_word(&mut self, index: u8, offset: u8, value: u16);

    /// Current `0x3B04A(stats)`. Firmware clears stats byte +15 before this call and
    /// then re-reads later fields, so mutations must remain observable.
    fn stats_boundary(&mut self, stats: &mut BtStage51Stats);
    /// Current `0x3D24(stats + 16, 0, 0x74)`. The exact runtime implementation remains
    /// opaque; the backend must expose any mutations of the represented stats fields.
    fn window_boundary(&mut self, stats: &mut BtStage51Stats);
}

const fn stage51_mode(byte90: u8) -> u8 {
    (byte90 >> 3) & 0x0F
}

const fn stage51_event_mode(byte0: u8) -> u8 {
    (byte0 >> 3) & 0x0F
}

fn stage51_inc_record_byte<B: BtStage51Backend>(
    backend: &mut B,
    index: u8,
    offset: u8,
) {
    let value = backend.record_byte(index, offset).wrapping_add(1);
    backend.set_record_byte(index, offset, value);
}

/// Safe source-level model of current `0x16DBDC` / legacy `sub_16AC10`.
///
/// The firmware does not present a stable semantic return value on all paths, so this
/// reconstruction models the routine as an effectful coordinator rather than assigning
/// meaning to incidental live contents of R0 at return.
pub fn bt_stage51_record_state_coordinator<B: BtStage51Backend>(
    object: &BtStage51ObjectState,
    stats: &mut BtStage51Stats,
    event: &BtStage51Event,
    backend: &mut B,
) {
    if backend.context_boundary(object.bytea4) == 0 {
        return;
    }

    if object.byte94 == 2 {
        if object.byte90 & 0x80 == 0 {
            stats.byte20 |= 0x02;
        }
        if stage51_mode(object.byte90) <= 2 {
            stats.byte22 = stats.byte22.wrapping_add(1);
        }
    }

    if event.byte6 != 0 {
        let event_class = event.byte2 & 0x03;
        if stage51_event_mode(event.byte0) > 2 && (event_class == 1 || event_class == 2) {
            let index = stats.byte1;
            if object.byte94 == 2 {
                if object.byte91 & 0x01 == 0 {
                    stage51_inc_record_byte(backend, index, STAGE51_RECORD_BYTE22);
                } else {
                    let sample = backend.sample_boundary() as u8;
                    backend.set_record_byte(index, STAGE51_RECORD_BYTE28, sample);
                    backend.set_record_byte(index, STAGE51_RECORD_BYTE29, object.byte113);
                }
            } else {
                stage51_inc_record_byte(backend, index, STAGE51_RECORD_BYTE21);
            }
        }

        if object.byte15 == 0 && stats.byte0 != 0 {
            let index = stats.byte1;
            let accumulated = backend
                .record_word(index, STAGE51_RECORD_WORD26)
                .wrapping_add(u16::from(event.byte4))
                .wrapping_add(u16::from(object.byte96));
            backend.set_record_word(index, STAGE51_RECORD_WORD26, accumulated);
            stats.byte0 = 0;
        }
    } else if object.byte15 == 1 && object.byte94 != 2 {
        let index = stats.byte1;
        stage51_inc_record_byte(backend, index, STAGE51_RECORD_BYTE23);
        if stats.byte0 != 0 {
            let accumulated = backend
                .record_word(index, STAGE51_RECORD_WORD26)
                .wrapping_add(2);
            backend.set_record_word(index, STAGE51_RECORD_WORD26, accumulated);
            stats.byte0 = event.byte6;
        }
    }

    if stage51_mode(object.byte90) > 1 {
        return;
    }

    if stats.byte15 != 0 {
        stats.byte15 = 0;
        backend.stats_boundary(stats);
    }

    if stats.byte14 != 0 {
        stats.byte14 = 0;
        backend.window_boundary(stats);
        stats.byte1 = 0;
        return;
    }

    let flags = stats.byte20;
    if flags & 0x01 == 0 {
        let limit = flags >> 5;
        if stats.byte1 < limit {
            stats.byte1 = stats.byte1.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod stage51_tests {
    extern crate std;
    use super::*;
    use std::collections::BTreeMap;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        context: u32,
        sample: u32,
        bytes: BTreeMap<(u8, u8), u8>,
        words: BTreeMap<(u8, u8), u16>,
        calls: Vec<(&'static str, u32, u32)>,
        stats_set_byte14: Option<u8>,
        stats_set_byte20: Option<u8>,
        window_seen_byte14: u8,
    }

    impl BtStage51Backend for B {
        fn context_boundary(&mut self, selector:u8)->u32 {
            self.calls.push(("context",selector as u32,0)); self.context
        }
        fn sample_boundary(&mut self)->u32 {
            self.calls.push(("sample",0,0)); self.sample
        }
        fn record_byte(&mut self,index:u8,offset:u8)->u8 {
            self.calls.push(("read_byte",index as u32,offset as u32));
            *self.bytes.get(&(index,offset)).unwrap_or(&0)
        }
        fn set_record_byte(&mut self,index:u8,offset:u8,value:u8) {
            self.calls.push(("write_byte",index as u32,offset as u32));
            self.bytes.insert((index,offset),value);
        }
        fn record_word(&mut self,index:u8,offset:u8)->u16 {
            self.calls.push(("read_word",index as u32,offset as u32));
            *self.words.get(&(index,offset)).unwrap_or(&0)
        }
        fn set_record_word(&mut self,index:u8,offset:u8,value:u16) {
            self.calls.push(("write_word",index as u32,offset as u32));
            self.words.insert((index,offset),value);
        }
        fn stats_boundary(&mut self,stats:&mut BtStage51Stats) {
            self.calls.push(("stats",stats.byte15 as u32,0));
            if let Some(v)=self.stats_set_byte14 { stats.byte14=v; }
            if let Some(v)=self.stats_set_byte20 { stats.byte20=v; }
        }
        fn window_boundary(&mut self,stats:&mut BtStage51Stats) {
            self.window_seen_byte14=stats.byte14;
            self.calls.push(("window",stats.byte1 as u32,0));
            stats.byte20=0;
            stats.byte22=0;
        }
    }

    fn object() -> BtStage51ObjectState {
        BtStage51ObjectState { byte15:0,byte90:0x18,byte91:0,byte94:0,byte96:3,bytea4:7,byte113:0xE1 }
    }
    fn stats() -> BtStage51Stats {
        BtStage51Stats { byte0:0,byte1:2,byte14:0,byte15:0,byte20:0,byte22:0 }
    }
    fn event() -> BtStage51Event {
        BtStage51Event { byte0:0x18,byte2:1,byte4:5,byte6:1 }
    }
    fn backend() -> B { B { context:0x1000,..Default::default() } }

    #[test]
    fn zero_context_is_a_strict_early_exit() {
        let o=object(); let mut s=stats(); let e=event(); let mut b=B::default();
        let before=s;
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(s,before);
        assert_eq!(b.calls,[("context",7,0)]);
    }

    #[test]
    fn state94_two_sets_flag_when_sign_bit_clear_and_counts_modes_zero_to_two() {
        let mut o=object(); o.byte94=2; o.byte90=0x10;
        let mut s=stats(); s.byte20=0x20; s.byte22=0xFF;
        let mut e=event(); e.byte6=0;
        let mut b=backend();
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(s.byte20,0x22);
        assert_eq!(s.byte22,0);
        o.byte90=0x90; s.byte20=0x20; s.byte22=9;
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(s.byte20,0x20);
        assert_eq!(s.byte22,10);
    }

    #[test]
    fn qualifying_event_updates_record22_for_state94_two_without_flag91() {
        let mut o=object(); o.byte94=2; o.byte90=0x20; o.byte91=0;
        let mut s=stats(); s.byte1=0xFE;
        let mut e=event(); e.byte0=0x18; e.byte2=2; e.byte6=1;
        let mut b=backend(); b.bytes.insert((0xFE,STAGE51_RECORD_BYTE22),0xFF);
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(b.bytes[&(0xFE,STAGE51_RECORD_BYTE22)],0);
    }

    #[test]
    fn state94_two_with_flag91_samples_byte28_and_copies_byte113_to29() {
        let mut o=object(); o.byte94=2; o.byte90=0x20; o.byte91=1; o.byte113=0xFE;
        let mut s=stats(); s.byte1=3;
        let e=event(); let mut b=backend(); b.sample=0x1234_ABCD;
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(b.bytes[&(3,STAGE51_RECORD_BYTE28)],0xCD);
        assert_eq!(b.bytes[&(3,STAGE51_RECORD_BYTE29)],0xFE);
        assert!(b.calls.iter().any(|x|x.0=="sample"));
    }

    #[test]
    fn non_state94_two_qualifying_event_counts_record21() {
        let o=object(); let mut s=stats(); s.byte1=4;
        let e=event(); let mut b=backend(); b.bytes.insert((4,STAGE51_RECORD_BYTE21),9);
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(b.bytes[&(4,STAGE51_RECORD_BYTE21)],10);
    }

    #[test]
    fn live_event_accumulates_word26_and_clears_stats_byte0() {
        let o=object(); let mut s=stats(); s.byte0=1; s.byte1=5;
        let e=event(); let mut b=backend(); b.words.insert((5,STAGE51_RECORD_WORD26),0xFFF9);
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(b.words[&(5,STAGE51_RECORD_WORD26)],1);
        assert_eq!(s.byte0,0);
    }

    #[test]
    fn zero_event_state15_one_counts_record23_and_adds_two() {
        let mut o=object(); o.byte15=1; o.byte94=1; o.byte90=0x20;
        let mut s=stats(); s.byte0=7; s.byte1=6;
        let mut e=event(); e.byte6=0;
        let mut b=backend(); b.bytes.insert((6,STAGE51_RECORD_BYTE23),0xFF); b.words.insert((6,STAGE51_RECORD_WORD26),0xFFFF);
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(b.bytes[&(6,STAGE51_RECORD_BYTE23)],0);
        assert_eq!(b.words[&(6,STAGE51_RECORD_WORD26)],1);
        assert_eq!(s.byte0,0);
    }

    #[test]
    fn mode_zero_rereads_byte14_after_stats_boundary_then_resets_index() {
        let mut o=object(); o.byte90=0;
        let mut s=stats(); s.byte15=1; s.byte14=0; s.byte1=7; s.byte20=0xE0;
        let mut e=event(); e.byte6=0;
        let mut b=backend(); b.stats_set_byte14=Some(1);
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!((s.byte15,s.byte14,s.byte1),(0,0,0));
        assert_eq!(b.window_seen_byte14,0);
        let a=b.calls.iter().position(|x|x.0=="stats").unwrap();
        let z=b.calls.iter().position(|x|x.0=="window").unwrap();
        assert!(a<z);
    }

    #[test]
    fn mode_one_advances_index_from_post_boundary_flags_only_when_allowed() {
        let mut o=object(); o.byte90=0x08;
        let mut s=stats(); s.byte15=1; s.byte1=2; s.byte20=0;
        let mut e=event(); e.byte6=0;
        let mut b=backend(); b.stats_set_byte20=Some(0x60);
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(s.byte1,3);
        s.byte15=0; s.byte1=1; s.byte20=0x61;
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!(s.byte1,1);
    }

    #[test]
    fn modes_above_one_skip_late_stats_boundaries() {
        let mut o=object(); o.byte90=0x18;
        let mut s=stats(); s.byte15=1; s.byte14=1;
        let mut e=event(); e.byte6=0;
        let mut b=backend();
        bt_stage51_record_state_coordinator(&o,&mut s,&e,&mut b);
        assert_eq!((s.byte15,s.byte14),(1,1));
        assert!(!b.calls.iter().any(|x|x.0=="stats" || x.0=="window"));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE51_CURRENT_BT_RECORD_COORDINATOR_ADDR,0x16DBDC);
        assert_eq!(STAGE51_BT_CONTEXT_BOUNDARY,0x335AC);
        assert_eq!(STAGE51_BT_SAMPLE_BOUNDARY,0x3A6CC);
        assert_eq!(STAGE51_BT_STATS_BOUNDARY,0x3B04A);
        assert_eq!(STAGE51_BT_WINDOW_BOUNDARY,0x3D24);
    }
}

/// Stage 52: current lookup/range gate at `0x16EBA4`.
///
/// The exact current 64-byte body is a relocation-normalized structural counterpart of
/// the 73136-byte legacy image's `sub_16BBD8`. Runtime calls and ambient bounds remain
/// opaque; this model preserves only the local argument flow, dword mask, call ordering,
/// short-circuiting, and open-interval comparison visible in current firmware.
pub const STAGE52_CURRENT_BT_LOOKUP_RANGE_GATE_ADDR: u32 = 0x0016_EBA4;
pub const STAGE52_BT_FIRST_BOUNDARY: u32 = 0x0003_38FC;
pub const STAGE52_BT_SECOND_BOUNDARY: u32 = 0x0004_D552;
pub const STAGE52_BT_TRANSFORM_BOUNDARY: u32 = 0x0001_8540;
pub const STAGE52_BT_LOWER_BOUND_ADDR: u32 = 0x0022_1EDC;
pub const STAGE52_BT_UPPER_BOUND_ADDR: u32 = 0x0022_1EE0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage52InputState {
    /// Input byte +0xA4, passed independently to the first and second opaque boundaries.
    pub bytea4: u8,
}

pub trait BtStage52Backend {
    /// Current `0x338FC(input.byteA4)`. A zero result is a strict local early exit.
    fn first_boundary(&mut self, selector: u8) -> u32;
    /// Dword +0 of the first-boundary object.
    fn first_dword0(&mut self, first: u32) -> u32;

    /// Current `0x4D552(input.byteA4)`.
    fn second_boundary(&mut self, selector: u8) -> u32;
    /// Dword +0x0C of the nonzero record obtained from `first_dword0`.
    fn record_dword12(&mut self, record: u32) -> u32;
    /// Current `0x18540(second_result, record_dword12 & 0x0FFF_FFFF)`.
    fn transform_boundary(&mut self, token: u32, masked_word: u32) -> u32;

    /// Current ambient dword loaded indirectly through literal `0x221EDC`.
    fn lower_bound(&mut self) -> u32;
    /// Current ambient dword loaded indirectly through literal `0x221EE0`.
    fn upper_bound(&mut self) -> u32;
}

/// Safe source-level model of current `0x16EBA4`.
///
/// The function returns one exactly when the locally produced value lies strictly inside
/// the firmware's ambient unsigned interval `(lower_bound, upper_bound)`. The upper bound
/// is deliberately not read when the lower comparison already rejects the value.
pub fn bt_stage52_lookup_range_gate<B: BtStage52Backend>(
    input: &BtStage52InputState,
    backend: &mut B,
) -> u32 {
    let first = backend.first_boundary(input.bytea4);
    if first == 0 {
        return 0;
    }

    let record = backend.first_dword0(first);
    let value = if record == 0 {
        0
    } else {
        let token = backend.second_boundary(input.bytea4);
        let masked_word = backend.record_dword12(record) & 0x0FFF_FFFF;
        backend.transform_boundary(token, masked_word)
    };

    let lower = backend.lower_bound();
    if value <= lower {
        return 0;
    }

    let upper = backend.upper_bound();
    u32::from(value < upper)
}

#[cfg(test)]
mod stage52_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        first: u32,
        record: u32,
        second: u32,
        word12: u32,
        transformed: u32,
        lower: u32,
        upper: u32,
        calls: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage52Backend for B {
        fn first_boundary(&mut self, selector: u8) -> u32 {
            self.calls.push(("first", selector as u32, 0));
            self.first
        }
        fn first_dword0(&mut self, first: u32) -> u32 {
            self.calls.push(("dword0", first, 0));
            self.record
        }
        fn second_boundary(&mut self, selector: u8) -> u32 {
            self.calls.push(("second", selector as u32, 0));
            self.second
        }
        fn record_dword12(&mut self, record: u32) -> u32 {
            self.calls.push(("dword12", record, 0));
            self.word12
        }
        fn transform_boundary(&mut self, token: u32, masked_word: u32) -> u32 {
            self.calls.push(("transform", token, masked_word));
            self.transformed
        }
        fn lower_bound(&mut self) -> u32 {
            self.calls.push(("lower", 0, 0));
            self.lower
        }
        fn upper_bound(&mut self) -> u32 {
            self.calls.push(("upper", 0, 0));
            self.upper
        }
    }

    fn input() -> BtStage52InputState {
        BtStage52InputState { bytea4: 7 }
    }

    #[test]
    fn zero_first_boundary_is_a_strict_early_exit() {
        let mut b = B::default();
        assert_eq!(bt_stage52_lookup_range_gate(&input(), &mut b), 0);
        assert_eq!(b.calls, [("first", 7, 0)]);
    }

    #[test]
    fn zero_record_skips_second_and_transform_and_rejects_at_lower_bound() {
        let mut b = B { first: 0x1000, record: 0, lower: 0, upper: 10, ..Default::default() };
        assert_eq!(bt_stage52_lookup_range_gate(&input(), &mut b), 0);
        assert_eq!(b.calls, [
            ("first", 7, 0),
            ("dword0", 0x1000, 0),
            ("lower", 0, 0),
        ]);
    }

    #[test]
    fn nonzero_record_preserves_call_order_and_clears_top_nibble() {
        let mut b = B {
            first: 0x1000,
            record: 0x2000,
            second: 0x3000,
            word12: 0xF123_4567,
            transformed: 15,
            lower: 10,
            upper: 20,
            ..Default::default()
        };
        assert_eq!(bt_stage52_lookup_range_gate(&input(), &mut b), 1);
        assert_eq!(b.calls, [
            ("first", 7, 0),
            ("dword0", 0x1000, 0),
            ("second", 7, 0),
            ("dword12", 0x2000, 0),
            ("transform", 0x3000, 0x0123_4567),
            ("lower", 0, 0),
            ("upper", 0, 0),
        ]);
    }

    #[test]
    fn lower_endpoint_is_excluded_without_reading_upper() {
        let mut b = B {
            first: 1,
            record: 2,
            second: 3,
            transformed: 10,
            lower: 10,
            upper: 20,
            ..Default::default()
        };
        assert_eq!(bt_stage52_lookup_range_gate(&input(), &mut b), 0);
        assert!(!b.calls.iter().any(|x| x.0 == "upper"));
    }

    #[test]
    fn upper_endpoint_is_excluded_but_strict_interior_is_accepted() {
        let mut b = B {
            first: 1,
            record: 2,
            second: 3,
            transformed: 20,
            lower: 10,
            upper: 20,
            ..Default::default()
        };
        assert_eq!(bt_stage52_lookup_range_gate(&input(), &mut b), 0);
        assert!(b.calls.iter().any(|x| x.0 == "upper"));

        b.calls.clear();
        b.transformed = 19;
        assert_eq!(bt_stage52_lookup_range_gate(&input(), &mut b), 1);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE52_CURRENT_BT_LOOKUP_RANGE_GATE_ADDR, 0x16EBA4);
        assert_eq!(STAGE52_BT_FIRST_BOUNDARY, 0x338FC);
        assert_eq!(STAGE52_BT_SECOND_BOUNDARY, 0x4D552);
        assert_eq!(STAGE52_BT_TRANSFORM_BOUNDARY, 0x18540);
        assert_eq!(STAGE52_BT_LOWER_BOUND_ADDR, 0x221EDC);
        assert_eq!(STAGE52_BT_UPPER_BOUND_ADDR, 0x221EE0);
    }
}

/// Stage 53: compact current post-gate object sequence at `0x16EEA4`.
///
/// The exact current 58-byte body is a relocation-normalized structural counterpart of
/// the public 73136-byte legacy image at `0x16BED8`. Runtime calls remain opaque; this
/// model preserves only the local argument flow, post-call rereads, ambient byte writes,
/// and return-value preservation visible in the current firmware.
pub const STAGE53_CURRENT_BT_POST_GATE_SEQUENCE_ADDR: u32 = 0x0016_EEA4;
pub const STAGE53_BT_GATE_BOUNDARY: u32 = 0x0002_1F20;
pub const STAGE53_BT_CONFIG_BOUNDARY: u32 = 0x0002_4824;
pub const STAGE53_BT_OBJECT_BOUNDARY_A: u32 = 0x0005_180C;
pub const STAGE53_BT_INTERNAL_BOUNDARY: u32 = 0x0016_F5F4;
pub const STAGE53_BT_FINAL_BOUNDARY: u32 = 0x0005_0B76;
pub const STAGE53_BT_AMBIENT_CLEAR_ADDR: u32 = 0x0020_A234;
pub const STAGE53_BT_AMBIENT_PUBLISH_ADDR: u32 = 0x0020_A223;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage53ObjectState {
    /// Object dword +0x1C, read only after the gate boundary returns nonzero.
    pub dword28: u32,
    /// Object dword +0x48, read only after the gate boundary returns nonzero.
    pub dword72: u32,
    /// Object byte +0x14, deliberately re-read after all object-pointer boundaries.
    pub byte20: u8,
    /// Object byte +0x5F, forced to one after the final opaque boundary.
    pub byte95: u8,
}

pub trait BtStage53Backend {
    /// Current `0x21F20(object)`. Zero is a strict local early exit; the backend may
    /// mutate object state before later dword reads.
    fn gate_boundary(&mut self, object: &mut BtStage53ObjectState) -> u32;

    /// Current `0x24824(object.dword72, object.dword28, 1)`.
    fn config_boundary(&mut self, first: u32, second: u32, enable: u32);

    /// Current ambient byte store through literal `0x20A234`.
    fn set_ambient_clear_byte(&mut self, value: u8);

    /// Current `0x5180C(object)`. Its return is not consumed locally.
    fn object_boundary_a(&mut self, object: &mut BtStage53ObjectState);
    /// Current internal `0x16F5F4(object)`. It remains opaque in Stage 53.
    fn internal_boundary(&mut self, object: &mut BtStage53ObjectState);
    /// Current `0x50B76(object)`. Its return value survives the subsequent local stores
    /// and is the routine's final return on the nonzero-gate path.
    fn final_boundary(&mut self, object: &mut BtStage53ObjectState) -> u32;

    /// Current ambient byte store through literal `0x20A223`.
    fn set_ambient_publish_byte(&mut self, value: u8);
}

/// Safe source-level model of current `0x16EEA4`.
pub fn bt_stage53_post_gate_sequence<B: BtStage53Backend>(
    object: &mut BtStage53ObjectState,
    backend: &mut B,
) -> u32 {
    let gate = backend.gate_boundary(object);
    if gate == 0 {
        return 0;
    }

    // These two fields are loaded after the gate call, so gate-side mutations must be
    // observable here. Firmware passes dword72 in R0, dword28 in R1, and literal 1 in R2.
    let second = object.dword28;
    let first = object.dword72;
    backend.config_boundary(first, second, 1);

    backend.set_ambient_clear_byte(0);
    backend.object_boundary_a(object);
    backend.internal_boundary(object);
    let final_result = backend.final_boundary(object);

    object.byte95 = 1;
    // The firmware reads byte20 only after all three object-pointer boundaries.
    let publish = object.byte20;
    backend.set_ambient_publish_byte(publish);

    final_result
}

#[cfg(test)]
mod stage53_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        gate: u32,
        final_result: u32,
        gate_dword28: Option<u32>,
        gate_dword72: Option<u32>,
        a_byte20: Option<u8>,
        internal_byte20: Option<u8>,
        final_byte20: Option<u8>,
        ambient_clear: Option<u8>,
        ambient_publish: Option<u8>,
        calls: Vec<(&'static str, u32, u32, u32)>,
    }

    impl BtStage53Backend for B {
        fn gate_boundary(&mut self, object: &mut BtStage53ObjectState) -> u32 {
            self.calls.push(("gate", 0, 0, 0));
            if let Some(v) = self.gate_dword28 { object.dword28 = v; }
            if let Some(v) = self.gate_dword72 { object.dword72 = v; }
            self.gate
        }
        fn config_boundary(&mut self, first: u32, second: u32, enable: u32) {
            self.calls.push(("config", first, second, enable));
        }
        fn set_ambient_clear_byte(&mut self, value: u8) {
            self.calls.push(("clear", value as u32, 0, 0));
            self.ambient_clear = Some(value);
        }
        fn object_boundary_a(&mut self, object: &mut BtStage53ObjectState) {
            self.calls.push(("object_a", 0, 0, 0));
            if let Some(v) = self.a_byte20 { object.byte20 = v; }
        }
        fn internal_boundary(&mut self, object: &mut BtStage53ObjectState) {
            self.calls.push(("internal", 0, 0, 0));
            if let Some(v) = self.internal_byte20 { object.byte20 = v; }
        }
        fn final_boundary(&mut self, object: &mut BtStage53ObjectState) -> u32 {
            self.calls.push(("final", 0, 0, 0));
            if let Some(v) = self.final_byte20 { object.byte20 = v; }
            self.final_result
        }
        fn set_ambient_publish_byte(&mut self, value: u8) {
            self.calls.push(("publish", value as u32, 0, 0));
            self.ambient_publish = Some(value);
        }
    }

    fn object() -> BtStage53ObjectState {
        BtStage53ObjectState { dword28: 0x1111, dword72: 0x2222, byte20: 3, byte95: 0 }
    }

    #[test]
    fn zero_gate_is_a_strict_early_exit() {
        let mut o = object();
        let before = o;
        let mut b = B::default();
        assert_eq!(bt_stage53_post_gate_sequence(&mut o, &mut b), 0);
        assert_eq!(o, before);
        assert_eq!(b.calls, [("gate", 0, 0, 0)]);
    }

    #[test]
    fn config_arguments_are_read_after_gate_mutation_and_in_binary_order() {
        let mut o = object();
        let mut b = B {
            gate: 1,
            final_result: 7,
            gate_dword28: Some(0xAAAA_BBBB),
            gate_dword72: Some(0xCCCC_DDDD),
            ..Default::default()
        };
        assert_eq!(bt_stage53_post_gate_sequence(&mut o, &mut b), 7);
        assert_eq!(b.calls[1], ("config", 0xCCCC_DDDD, 0xAAAA_BBBB, 1));
    }

    #[test]
    fn ambient_clear_precedes_object_boundaries_and_is_exact_zero() {
        let mut o = object();
        let mut b = B { gate: 1, final_result: 9, ..Default::default() };
        let _ = bt_stage53_post_gate_sequence(&mut o, &mut b);
        assert_eq!(b.ambient_clear, Some(0));
        let clear = b.calls.iter().position(|x| x.0 == "clear").unwrap();
        let a = b.calls.iter().position(|x| x.0 == "object_a").unwrap();
        let internal = b.calls.iter().position(|x| x.0 == "internal").unwrap();
        let final_call = b.calls.iter().position(|x| x.0 == "final").unwrap();
        assert!(clear < a && a < internal && internal < final_call);
    }

    #[test]
    fn final_return_survives_local_stores_and_publish_uses_post_call_byte20() {
        let mut o = object();
        let mut b = B {
            gate: 1,
            final_result: 0xDEAD_BEEF,
            a_byte20: Some(0x11),
            internal_byte20: Some(0x22),
            final_byte20: Some(0x33),
            ..Default::default()
        };
        assert_eq!(bt_stage53_post_gate_sequence(&mut o, &mut b), 0xDEAD_BEEF);
        assert_eq!(o.byte95, 1);
        assert_eq!(o.byte20, 0x33);
        assert_eq!(b.ambient_publish, Some(0x33));
        assert_eq!(b.calls.last().copied(), Some(("publish", 0x33, 0, 0)));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE53_CURRENT_BT_POST_GATE_SEQUENCE_ADDR, 0x16EEA4);
        assert_eq!(STAGE53_BT_GATE_BOUNDARY, 0x21F20);
        assert_eq!(STAGE53_BT_CONFIG_BOUNDARY, 0x24824);
        assert_eq!(STAGE53_BT_OBJECT_BOUNDARY_A, 0x5180C);
        assert_eq!(STAGE53_BT_INTERNAL_BOUNDARY, 0x16F5F4);
        assert_eq!(STAGE53_BT_FINAL_BOUNDARY, 0x50B76);
        assert_eq!(STAGE53_BT_AMBIENT_CLEAR_ADDR, 0x20A234);
        assert_eq!(STAGE53_BT_AMBIENT_PUBLISH_ADDR, 0x20A223);
    }
}
