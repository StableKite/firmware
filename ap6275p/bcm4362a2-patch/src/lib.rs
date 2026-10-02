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

/// Stage 54: compact current two-boundary wrapper at `0x16F228`.
///
/// The exact current 20-byte body is a relocation-normalized structural counterpart of
/// the public 73136-byte legacy image at `0x16C25C`. Both runtime targets remain opaque;
/// this model preserves only the literal argument, object-token forwarding, call order,
/// and tail-return behavior visible in current firmware.
pub const STAGE54_CURRENT_BT_TWO_BOUNDARY_WRAPPER_ADDR: u32 = 0x0016_F228;
pub const STAGE54_BT_FIRST_BOUNDARY: u32 = 0x0004_4468;
pub const STAGE54_BT_TAIL_BOUNDARY: u32 = 0x0002_65E8;

pub trait BtStage54Backend {
    /// Current `0x44468(object, 1)`. The return value is ignored locally.
    fn first_boundary(&mut self, object: u32, enable: u32);
    /// Current tail `0x265E8(object)`. Its return is the wrapper's final return.
    fn tail_boundary(&mut self, object: u32) -> u32;
}

/// Safe source-level model of current `0x16F228`.
///
/// The firmware performs no local null/range check on the opaque object token.
pub fn bt_stage54_two_boundary_wrapper<B: BtStage54Backend>(
    object: u32,
    backend: &mut B,
) -> u32 {
    backend.first_boundary(object, 1);
    backend.tail_boundary(object)
}

#[cfg(test)]
mod stage54_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        tail_result: u32,
        calls: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage54Backend for B {
        fn first_boundary(&mut self, object: u32, enable: u32) {
            self.calls.push(("first", object, enable));
        }
        fn tail_boundary(&mut self, object: u32) -> u32 {
            self.calls.push(("tail", object, 0));
            self.tail_result
        }
    }

    #[test]
    fn forwards_same_object_with_literal_one_then_tail_calls() {
        let mut b = B { tail_result: 0xDEAD_BEEF, ..Default::default() };
        assert_eq!(bt_stage54_two_boundary_wrapper(0x1234_5678, &mut b), 0xDEAD_BEEF);
        assert_eq!(b.calls, [
            ("first", 0x1234_5678, 1),
            ("tail", 0x1234_5678, 0),
        ]);
    }

    #[test]
    fn zero_object_is_forwarded_without_inventing_a_local_guard() {
        let mut b = B { tail_result: 7, ..Default::default() };
        assert_eq!(bt_stage54_two_boundary_wrapper(0, &mut b), 7);
        assert_eq!(b.calls, [("first", 0, 1), ("tail", 0, 0)]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE54_CURRENT_BT_TWO_BOUNDARY_WRAPPER_ADDR, 0x16F228);
        assert_eq!(STAGE54_BT_FIRST_BOUNDARY, 0x44468);
        assert_eq!(STAGE54_BT_TAIL_BOUNDARY, 0x265E8);
    }
}

/// Stage 55: compact current masked-record scan at `0x16F306`.
///
/// The exact current 40-byte body is byte-identical to the public 73136-byte legacy
/// structural counterpart at `0x16C33A`. There are no direct calls in the body.
/// This model preserves the single mask load, low-three-bit scan order, exact table
/// stride/status offset, first-match return, and zero fallback visible in current firmware.
pub const STAGE55_CURRENT_BT_MASKED_RECORD_SCAN_ADDR: u32 = 0x0016_F306;
pub const STAGE55_BT_RECORD_TABLE_BASE_ADDR: u32 = 0x0020_A2D4;
pub const STAGE55_BT_RECORD_STRIDE: u32 = 0x84;
pub const STAGE55_BT_STATUS_HALFWORD_OFFSET: u32 = 0x22;
pub const STAGE55_BT_MATCH_STATUS: u16 = 6;
pub const STAGE55_BT_RECORD_COUNT: u32 = 3;

pub trait BtStage55Backend {
    /// Reads the dword pointed to by incoming R3. Current firmware performs this load once,
    /// before the scan index is initialized.
    fn read_mask_dword(&mut self, mask_ptr: u32) -> u32;

    /// Reads the halfword at `record + 0x22`.
    fn read_record_status_halfword(&mut self, record: u32) -> u16;
}

/// Safe source-level model of current `0x16F306`.
///
/// Only indices 0, 1, and 2 are scanned. A record is inspected only when its corresponding
/// bit is set in the single mask snapshot. The first selected record whose status halfword
/// equals six is returned as its current firmware address; otherwise zero is returned.
pub fn bt_stage55_masked_record_scan<B: BtStage55Backend>(
    mask_ptr: u32,
    backend: &mut B,
) -> u32 {
    let mask = backend.read_mask_dword(mask_ptr);

    let mut index = 0u32;
    while index < STAGE55_BT_RECORD_COUNT {
        let bit = 1u32 << index;
        if (mask & bit) != 0 {
            let record = STAGE55_BT_RECORD_TABLE_BASE_ADDR
                .wrapping_add(STAGE55_BT_RECORD_STRIDE.wrapping_mul(index));
            if backend.read_record_status_halfword(record) == STAGE55_BT_MATCH_STATUS {
                return record;
            }
        }
        index += 1;
    }

    0
}

#[cfg(test)]
mod stage55_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        mask: u32,
        statuses: [u16; 3],
        mask_reads: u32,
        status_reads: Vec<u32>,
    }

    impl BtStage55Backend for B {
        fn read_mask_dword(&mut self, _mask_ptr: u32) -> u32 {
            self.mask_reads += 1;
            self.mask
        }

        fn read_record_status_halfword(&mut self, record: u32) -> u16 {
            self.status_reads.push(record);
            let index = (record - STAGE55_BT_RECORD_TABLE_BASE_ADDR) / STAGE55_BT_RECORD_STRIDE;
            self.statuses[index as usize]
        }
    }

    fn record(index: u32) -> u32 {
        STAGE55_BT_RECORD_TABLE_BASE_ADDR + STAGE55_BT_RECORD_STRIDE * index
    }

    #[test]
    fn mask_is_loaded_once_and_zero_mask_reads_no_records() {
        let mut b = B::default();
        assert_eq!(bt_stage55_masked_record_scan(0x1234_5678, &mut b), 0);
        assert_eq!(b.mask_reads, 1);
        assert!(b.status_reads.is_empty());
    }

    #[test]
    fn unselected_records_are_skipped_and_first_selected_match_returns() {
        let mut b = B {
            mask: 0b110,
            statuses: [6, 6, 6],
            ..Default::default()
        };
        assert_eq!(bt_stage55_masked_record_scan(1, &mut b), record(1));
        assert_eq!(b.mask_reads, 1);
        assert_eq!(b.status_reads, [record(1)]);
    }

    #[test]
    fn selected_nonmatch_continues_to_later_selected_match() {
        let mut b = B {
            mask: 0b111,
            statuses: [5, 7, 6],
            ..Default::default()
        };
        assert_eq!(bt_stage55_masked_record_scan(2, &mut b), record(2));
        assert_eq!(b.status_reads, [record(0), record(1), record(2)]);
    }

    #[test]
    fn bits_above_two_are_ignored() {
        let mut b = B {
            mask: 0xFFFF_FFF8,
            statuses: [6, 6, 6],
            ..Default::default()
        };
        assert_eq!(bt_stage55_masked_record_scan(3, &mut b), 0);
        assert!(b.status_reads.is_empty());
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE55_CURRENT_BT_MASKED_RECORD_SCAN_ADDR, 0x16F306);
        assert_eq!(STAGE55_BT_RECORD_TABLE_BASE_ADDR, 0x20A2D4);
        assert_eq!(STAGE55_BT_RECORD_STRIDE, 0x84);
        assert_eq!(STAGE55_BT_STATUS_HALFWORD_OFFSET, 0x22);
        assert_eq!(STAGE55_BT_MATCH_STATUS, 6);
        assert_eq!(STAGE55_BT_RECORD_COUNT, 3);
    }
}

/// Stage 56: current gated bit-22 update at `0x16F438`.
pub const STAGE56_CURRENT_BT_GATED_BIT22_UPDATE_ADDR: u32 = 0x0016_F438;
pub const STAGE56_BT_PROBE_BOUNDARY: u32 = 0x0002_1DE8;
pub const STAGE56_BT_TRIPLET_BASE_ADDR: u32 = 0x0022_1F1D;
pub const STAGE56_BT_FLAG_ADDR: u32 = 0x0022_1F1C;
pub const STAGE56_BT_OUTPUT_WORD_ADDR: u32 = 0x0020_9644;
pub const STAGE56_BT_OUTPUT_BIT: u32 = 1 << 22;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage56ObjectState {
    pub byte15: u8,
    pub halfword34: u16,
    pub byte46: u8,
    pub byte52: u8,
}

pub trait BtStage56Backend {
    fn probe_boundary(&mut self, object: Option<&mut BtStage56ObjectState>, selector: u32) -> u32;
    fn read_triplet_byte(&mut self, offset: u32) -> u8;
    fn read_flag_byte(&mut self) -> u8;
    fn read_output_word(&mut self) -> u32;
    fn write_output_word(&mut self, value: u32);
}

/// Preserves the post-probe gates, signed threshold compare, and exact bit-22 RMW.
pub fn bt_stage56_gated_bit22_update<B: BtStage56Backend>(
    mut object: Option<&mut BtStage56ObjectState>,
    mode: u32,
    backend: &mut B,
) -> u32 {
    let probe = backend.probe_boundary(object.as_deref_mut(), 2);
    let Some(object) = object else { return probe };

    let seed = if object.byte15 != 0 {
        if mode != 1 { return probe; }
        0
    } else if object.byte52 != 0 {
        u32::from(object.byte52)
    } else {
        if mode != 1 { return probe; }
        0
    };

    let triplet2 = backend.read_triplet_byte(2);
    let triplet1 = backend.read_triplet_byte(1);
    let mut bit = false;
    if triplet2 < triplet1 {
        let sum = u32::from(object.byte52).wrapping_add(u32::from(object.byte46));
        let scale = u32::from(backend.read_triplet_byte(0));
        let threshold = scale.wrapping_mul(sum).wrapping_add(seed);
        if (probe as i32) < (threshold as i32) {
            let flag = backend.read_flag_byte();
            bit = flag == 0 || object.halfword34 != 6;
        }
    }

    let old = backend.read_output_word();
    backend.write_output_word((old & !STAGE56_BT_OUTPUT_BIT) | (u32::from(bit) << 22));
    probe
}

#[cfg(test)]
mod stage56_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        probe: u32,
        mutate15: Option<u8>,
        triplet: [u8; 3],
        flag: u8,
        output: u32,
        calls: Vec<(&'static str, u32)>,
    }

    impl BtStage56Backend for B {
        fn probe_boundary(&mut self, object: Option<&mut BtStage56ObjectState>, selector: u32) -> u32 {
            self.calls.push(("probe", selector));
            if let (Some(object), Some(v)) = (object, self.mutate15) { object.byte15 = v; }
            self.probe
        }
        fn read_triplet_byte(&mut self, offset: u32) -> u8 {
            self.calls.push(("triplet", offset));
            self.triplet[offset as usize]
        }
        fn read_flag_byte(&mut self) -> u8 {
            self.calls.push(("flag", 0));
            self.flag
        }
        fn read_output_word(&mut self) -> u32 {
            self.calls.push(("read_output", 0));
            self.output
        }
        fn write_output_word(&mut self, value: u32) {
            self.calls.push(("write_output", value));
            self.output = value;
        }
    }

    fn object() -> BtStage56ObjectState {
        BtStage56ObjectState { byte15: 0, halfword34: 5, byte46: 4, byte52: 3 }
    }

    #[test]
    fn early_paths_preserve_probe_return_and_skip_output() {
        let mut b = B { probe: 0xDEAD_BEEF, ..Default::default() };
        assert_eq!(bt_stage56_gated_bit22_update(None, 9, &mut b), 0xDEAD_BEEF);
        assert_eq!(b.calls, [("probe", 2)]);
        let mut o = object();
        b.calls.clear(); b.mutate15 = Some(1);
        assert_eq!(bt_stage56_gated_bit22_update(Some(&mut o), 0, &mut b), 0xDEAD_BEEF);
        assert_eq!(b.calls, [("probe", 2)]);
    }

    #[test]
    fn threshold_path_uses_seed_and_sets_bit22() {
        let mut o = object();
        let mut b = B { probe: 16, triplet: [2, 10, 5], flag: 0, ..Default::default() };
        assert_eq!(bt_stage56_gated_bit22_update(Some(&mut o), 99, &mut b), 16);
        assert_eq!(b.output, STAGE56_BT_OUTPUT_BIT);
        assert_eq!(b.calls, [
            ("probe", 2), ("triplet", 2), ("triplet", 1), ("triplet", 0),
            ("flag", 0), ("read_output", 0), ("write_output", STAGE56_BT_OUTPUT_BIT),
        ]);
    }

    #[test]
    fn signed_compare_and_halfword_gate_preserve_only_bit22() {
        let mut o = object();
        o.halfword34 = 6;
        let keep = 0xA5BF_FF5A;
        let mut b = B { probe: 0xFFFF_FFFF, triplet: [1, 2, 1], flag: 1,
                        output: keep | STAGE56_BT_OUTPUT_BIT, ..Default::default() };
        assert_eq!(bt_stage56_gated_bit22_update(Some(&mut o), 2, &mut b), 0xFFFF_FFFF);
        assert_eq!(b.output, keep & !STAGE56_BT_OUTPUT_BIT);
        o.halfword34 = 7; b.output = keep & !STAGE56_BT_OUTPUT_BIT;
        let _ = bt_stage56_gated_bit22_update(Some(&mut o), 2, &mut b);
        assert_eq!(b.output, (keep & !STAGE56_BT_OUTPUT_BIT) | STAGE56_BT_OUTPUT_BIT);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE56_CURRENT_BT_GATED_BIT22_UPDATE_ADDR, 0x16F438);
        assert_eq!(STAGE56_BT_PROBE_BOUNDARY, 0x21DE8);
        assert_eq!(STAGE56_BT_TRIPLET_BASE_ADDR, 0x221F1D);
        assert_eq!(STAGE56_BT_FLAG_ADDR, 0x221F1C);
        assert_eq!(STAGE56_BT_OUTPUT_WORD_ADDR, 0x209644);
        assert_eq!(STAGE56_BT_OUTPUT_BIT, 1 << 22);
    }
}

/// Stage 57: current gated counter/copy wrapper at `0x16F59E`.
///
/// The exact current 64-byte body is byte-identical to the public 73136-byte legacy
/// structural counterpart at `0x16C5D2`. The one direct call targets current Stage 56.
/// This source model preserves the input gates, signed byte comparison, wrapping ambient
/// byte update, low-six-bit source gate, post-Stage56 dword reads, copy order, and return flow.
pub const STAGE57_CURRENT_BT_COUNTER_COPY_ADDR: u32 = 0x0016_F59E;
pub const STAGE57_BT_STAGE56_BOUNDARY: u32 = 0x0016_F438;
pub const STAGE57_BT_TRIPLET_BASE_ADDR: u32 = 0x0022_1F1D;
pub const STAGE57_BT_SOURCE_BASE_ADDR: u32 = 0x0020_9644;
pub const STAGE57_BT_DEST_BASE_ADDR: u32 = 0x0065_0160;
pub const STAGE57_BT_SOURCE_GATE_MASK: u16 = 0x003F;
pub const STAGE57_BT_SOURCE_GATE_VALUE: u16 = 0x0019;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage57ObjectState {
    /// Object byte +0x6C, loaded unsigned.
    pub byte108: u8,
    /// Object byte +0x6E, loaded signed with LDRSB.
    pub signed_byte110: i8,
}

pub trait BtStage57Backend {
    /// Current ambient byte at `*(0x221F1D + 2)`.
    fn read_triplet2(&mut self) -> u8;
    fn write_triplet2(&mut self, value: u8);

    /// Halfword at current source base +2 (`0x209646`).
    fn read_source_halfword2(&mut self) -> u16;

    /// Current Stage-56 `0x16F438(object, 1)` boundary.
    fn stage56_boundary(&mut self, object: u32, mode: u32) -> u32;

    /// Reads current source dword at offset 0 or 4 from `0x209644`.
    fn read_source_dword(&mut self, offset: u32) -> u32;
    /// Writes current destination dword at offset 0 or 4 from `0x650160`.
    fn write_dest_dword(&mut self, offset: u32, value: u32);
}

/// Safe source-level model of current `0x16F59E`.
///
/// When `gate` is zero the incoming object token is returned unchanged and no memory is
/// touched. On the Stage-56 path, the two source dwords are deliberately read *after* the
/// Stage-56 call, matching the firmware ordering.
pub fn bt_stage57_counter_copy<B: BtStage57Backend>(
    object: u32,
    state: &BtStage57ObjectState,
    gate: u32,
    flags: u32,
    backend: &mut B,
) -> u32 {
    if gate == 0 {
        return object;
    }

    let next_triplet2 = if (flags & 1) != 0
        && i32::from(state.signed_byte110) > i32::from(state.byte108)
    {
        backend.read_triplet2().wrapping_add(1)
    } else {
        0
    };
    backend.write_triplet2(next_triplet2);

    if backend.read_source_halfword2() & STAGE57_BT_SOURCE_GATE_MASK
        != STAGE57_BT_SOURCE_GATE_VALUE
    {
        return object;
    }

    let result = backend.stage56_boundary(object, 1);
    let word0 = backend.read_source_dword(0);
    backend.write_dest_dword(0, word0);
    let word4 = backend.read_source_dword(4);
    backend.write_dest_dword(4, word4);
    result
}

#[cfg(test)]
mod stage57_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        triplet2: u8,
        halfword2: u16,
        source: [u32; 2],
        stage56_result: u32,
        stage56_post_source: Option<[u32; 2]>,
        dest: [u32; 2],
        calls: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage57Backend for B {
        fn read_triplet2(&mut self) -> u8 {
            self.calls.push(("read_triplet2", 0, 0));
            self.triplet2
        }
        fn write_triplet2(&mut self, value: u8) {
            self.calls.push(("write_triplet2", value as u32, 0));
            self.triplet2 = value;
        }
        fn read_source_halfword2(&mut self) -> u16 {
            self.calls.push(("halfword2", 0, 0));
            self.halfword2
        }
        fn stage56_boundary(&mut self, object: u32, mode: u32) -> u32 {
            self.calls.push(("stage56", object, mode));
            if let Some(words) = self.stage56_post_source {
                self.source = words;
            }
            self.stage56_result
        }
        fn read_source_dword(&mut self, offset: u32) -> u32 {
            self.calls.push(("read_source", offset, 0));
            self.source[(offset / 4) as usize]
        }
        fn write_dest_dword(&mut self, offset: u32, value: u32) {
            self.calls.push(("write_dest", offset, value));
            self.dest[(offset / 4) as usize] = value;
        }
    }

    #[test]
    fn zero_gate_returns_object_without_memory_access() {
        let state = BtStage57ObjectState { byte108: 1, signed_byte110: 2 };
        let mut b = B::default();
        assert_eq!(bt_stage57_counter_copy(0x1234_5678, &state, 0, 1, &mut b), 0x1234_5678);
        assert!(b.calls.is_empty());
    }

    #[test]
    fn signed_compare_controls_wrapping_counter_increment() {
        let state = BtStage57ObjectState { byte108: 100, signed_byte110: 101 };
        let mut b = B { triplet2: 0xFF, halfword2: 0, ..Default::default() };
        assert_eq!(bt_stage57_counter_copy(7, &state, 1, 1, &mut b), 7);
        assert_eq!(b.triplet2, 0);
        assert_eq!(b.calls[0], ("read_triplet2", 0, 0));

        let negative = BtStage57ObjectState { byte108: 0, signed_byte110: -1 };
        b.calls.clear(); b.triplet2 = 9;
        let _ = bt_stage57_counter_copy(7, &negative, 1, 1, &mut b);
        assert_eq!(b.triplet2, 0);
        assert!(!b.calls.iter().any(|x| x.0 == "read_triplet2"));
    }

    #[test]
    fn clear_flag_forces_counter_zero_and_failed_gate_preserves_object_return() {
        let state = BtStage57ObjectState { byte108: 0, signed_byte110: 100 };
        let mut b = B { triplet2: 7, halfword2: 0x0018, ..Default::default() };
        assert_eq!(bt_stage57_counter_copy(0x55AA, &state, 1, 0, &mut b), 0x55AA);
        assert_eq!(b.triplet2, 0);
        assert!(!b.calls.iter().any(|x| x.0 == "stage56"));
    }

    #[test]
    fn low_six_bit_gate_calls_stage56_then_copies_post_call_words_in_order() {
        let state = BtStage57ObjectState { byte108: 4, signed_byte110: 5 };
        let mut b = B {
            triplet2: 8,
            halfword2: 0xFFD9, // low six bits == 0x19
            source: [0x1111_1111, 0x2222_2222],
            stage56_result: 0xDEAD_BEEF,
            stage56_post_source: Some([0xAAAA_AAAA, 0xBBBB_BBBB]),
            ..Default::default()
        };
        assert_eq!(bt_stage57_counter_copy(0xCAFE, &state, 1, 1, &mut b), 0xDEAD_BEEF);
        assert_eq!(b.dest, [0xAAAA_AAAA, 0xBBBB_BBBB]);
        let stage56 = b.calls.iter().position(|x| x.0 == "stage56").unwrap();
        let read0 = b.calls.iter().position(|x| x.0 == "read_source" && x.1 == 0).unwrap();
        let write0 = b.calls.iter().position(|x| x.0 == "write_dest" && x.1 == 0).unwrap();
        let read4 = b.calls.iter().position(|x| x.0 == "read_source" && x.1 == 4).unwrap();
        let write4 = b.calls.iter().position(|x| x.0 == "write_dest" && x.1 == 4).unwrap();
        assert!(stage56 < read0 && read0 < write0 && write0 < read4 && read4 < write4);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE57_CURRENT_BT_COUNTER_COPY_ADDR, 0x16F59E);
        assert_eq!(STAGE57_BT_STAGE56_BOUNDARY, 0x16F438);
        assert_eq!(STAGE57_BT_TRIPLET_BASE_ADDR, 0x221F1D);
        assert_eq!(STAGE57_BT_SOURCE_BASE_ADDR, 0x209644);
        assert_eq!(STAGE57_BT_DEST_BASE_ADDR, 0x650160);
        assert_eq!(STAGE57_BT_SOURCE_GATE_MASK, 0x3F);
        assert_eq!(STAGE57_BT_SOURCE_GATE_VALUE, 0x19);
    }
}

/// Stage 58: current ambient triplet-byte clear at `0x16F5F4`.
///
/// The exact current 8-byte leaf body is byte-identical to the public 73136-byte legacy
/// structural counterpart at `0x16C628`. The only relocation is the PC-relative literal
/// following the body: current points at `0x221F1D`, public legacy at older `0x221EE9`.
pub const STAGE58_CURRENT_BT_TRIPLET_CLEAR_ADDR: u32 = 0x0016_F5F4;
pub const STAGE58_BT_TRIPLET_BASE_ADDR: u32 = 0x0022_1F1D;
pub const STAGE58_BT_CLEARED_OFFSET: u32 = 2;

pub trait BtStage58Backend {
    /// Stores a byte at current `*(0x221F1D + 2)`.
    fn write_triplet2(&mut self, value: u8);
}

/// Safe source-level model of current `0x16F5F4`.
pub fn bt_stage58_clear_triplet2<B: BtStage58Backend>(backend: &mut B) {
    backend.write_triplet2(0);
}

#[cfg(test)]
mod stage58_tests {
    use super::*;

    #[derive(Default)]
    struct B {
        value: u8,
        writes: u32,
    }

    impl BtStage58Backend for B {
        fn write_triplet2(&mut self, value: u8) {
            self.value = value;
            self.writes += 1;
        }
    }

    #[test]
    fn writes_exact_zero_once() {
        let mut b = B { value: 0xA5, writes: 0 };
        bt_stage58_clear_triplet2(&mut b);
        assert_eq!(b.value, 0);
        assert_eq!(b.writes, 1);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE58_CURRENT_BT_TRIPLET_CLEAR_ADDR, 0x16F5F4);
        assert_eq!(STAGE58_BT_TRIPLET_BASE_ADDR, 0x221F1D);
        assert_eq!(STAGE58_BT_CLEARED_OFFSET, 2);
    }
}

/// Stage 59: current ambient-byte copy and callback publication at `0x16F600`.
///
/// The exact current 30-byte leaf body is byte-identical to the public 73136-byte legacy
/// structural counterpart at `0x16C634`. All addresses below are preserved as raw firmware
/// values; no semantic role is assigned beyond the local loads/stores visible in the body.
pub const STAGE59_CURRENT_BT_CALLBACK_PUBLISH_ADDR: u32 = 0x0016_F600;
pub const STAGE59_BT_SOURCE_BYTE_ADDR: u32 = 0x0020_A22A;
pub const STAGE59_BT_DEST_BYTE_ADDR: u32 = 0x0022_2709;
pub const STAGE59_BT_OBJECT_PTR_ADDR: u32 = 0x0020_2A74;
pub const STAGE59_BT_GLOBAL_BLOCK_ADDR: u32 = 0x0021_67D4;
pub const STAGE59_BT_GLOBAL_SLOT54_ADDR: u32 = 0x0021_6828;
pub const STAGE59_BT_GLOBAL_SLOT5C_ADDR: u32 = 0x0021_6830;
pub const STAGE59_BT_CALLBACK_A_THUMB: u32 = 0x0016_F511;
pub const STAGE59_BT_CALLBACK_B_THUMB: u32 = 0x0016_F4A9;
pub const STAGE59_BT_CALLBACK_C_THUMB: u32 = 0x0016_F33D;

pub trait BtStage59Backend {
    fn read_source_byte(&mut self) -> u8;
    fn write_dest_byte(&mut self, value: u8);
    fn read_object_ptr(&mut self) -> u32;
    fn write_global_slot54(&mut self, value: u32);
    fn write_object_slot14(&mut self, object: u32, value: u32);
    fn write_global_slot5c(&mut self, value: u32);
}

/// Safe source-level model of current `0x16F600`.
///
/// R0 is not modified by the firmware body, so the model returns the incoming token to make
/// that register-preservation property explicit.
pub fn bt_stage59_publish_callbacks<B: BtStage59Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    let byte = backend.read_source_byte();
    backend.write_dest_byte(byte);

    let object = backend.read_object_ptr();
    if object != 0 {
        backend.write_global_slot54(STAGE59_BT_CALLBACK_A_THUMB);
        backend.write_object_slot14(object, STAGE59_BT_CALLBACK_B_THUMB);
        backend.write_global_slot5c(STAGE59_BT_CALLBACK_C_THUMB);
    }

    incoming_r0
}

#[cfg(test)]
mod stage59_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        source_byte: u8,
        dest_byte: u8,
        object: u32,
        calls: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage59Backend for B {
        fn read_source_byte(&mut self) -> u8 {
            self.calls.push(("read_byte", 0, 0));
            self.source_byte
        }
        fn write_dest_byte(&mut self, value: u8) {
            self.calls.push(("write_byte", value as u32, 0));
            self.dest_byte = value;
        }
        fn read_object_ptr(&mut self) -> u32 {
            self.calls.push(("read_object", 0, 0));
            self.object
        }
        fn write_global_slot54(&mut self, value: u32) {
            self.calls.push(("global54", value, 0));
        }
        fn write_object_slot14(&mut self, object: u32, value: u32) {
            self.calls.push(("object14", object, value));
        }
        fn write_global_slot5c(&mut self, value: u32) {
            self.calls.push(("global5c", value, 0));
        }
    }

    #[test]
    fn byte_copy_always_occurs_and_zero_object_skips_callback_writes() {
        let mut b = B { source_byte: 0xA5, object: 0, ..Default::default() };
        assert_eq!(bt_stage59_publish_callbacks(0xCAFE_BABE, &mut b), 0xCAFE_BABE);
        assert_eq!(b.dest_byte, 0xA5);
        assert_eq!(b.calls, [
            ("read_byte", 0, 0),
            ("write_byte", 0xA5, 0),
            ("read_object", 0, 0),
        ]);
    }

    #[test]
    fn nonzero_object_publishes_three_raw_thumb_pointers_in_binary_order() {
        let mut b = B { source_byte: 7, object: 0x1234_5000, ..Default::default() };
        assert_eq!(bt_stage59_publish_callbacks(9, &mut b), 9);
        assert_eq!(b.calls, [
            ("read_byte", 0, 0),
            ("write_byte", 7, 0),
            ("read_object", 0, 0),
            ("global54", STAGE59_BT_CALLBACK_A_THUMB, 0),
            ("object14", 0x1234_5000, STAGE59_BT_CALLBACK_B_THUMB),
            ("global5c", STAGE59_BT_CALLBACK_C_THUMB, 0),
        ]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE59_CURRENT_BT_CALLBACK_PUBLISH_ADDR, 0x16F600);
        assert_eq!(STAGE59_BT_SOURCE_BYTE_ADDR, 0x20A22A);
        assert_eq!(STAGE59_BT_DEST_BYTE_ADDR, 0x222709);
        assert_eq!(STAGE59_BT_OBJECT_PTR_ADDR, 0x202A74);
        assert_eq!(STAGE59_BT_GLOBAL_BLOCK_ADDR, 0x2167D4);
        assert_eq!(STAGE59_BT_GLOBAL_SLOT54_ADDR, 0x216828);
        assert_eq!(STAGE59_BT_GLOBAL_SLOT5C_ADDR, 0x216830);
        assert_eq!(STAGE59_BT_CALLBACK_A_THUMB, 0x16F511);
        assert_eq!(STAGE59_BT_CALLBACK_B_THUMB, 0x16F4A9);
        assert_eq!(STAGE59_BT_CALLBACK_C_THUMB, 0x16F33D);
    }
}

/// Stage 60: current gated ten-byte saturating update at `0x16F8D0`.
///
/// The exact current 72-byte leaf body is byte-identical to the public 73136-byte legacy
/// structural counterpart at `0x16C670`. There are no direct runtime calls. The model
/// preserves the global gate, selector arithmetic, intentionally overlapping source window,
/// ten target writes, branch-specific saturation rule, and observable R0 return shape.
pub const STAGE60_CURRENT_BT_SATURATING_UPDATE_ADDR: u32 = 0x0016_F8D0;
pub const STAGE60_BT_GATE_ADDR: u32 = 0x0022_2747;
pub const STAGE60_BT_SELECTOR_BASE_ADDR: u32 = 0x0020_DCD2;
pub const STAGE60_BT_SELECTOR_OFFSET: u32 = 3;
pub const STAGE60_BT_SOURCE_BASE_ADDR: u32 = 0x0022_270B;
pub const STAGE60_BT_SOURCE_WINDOW_STRIDE: u32 = 10;
pub const STAGE60_BT_TARGET_BASE_ADDR: u32 = 0x0020_DE31;
pub const STAGE60_BT_TARGET_STRIDE: u32 = 0x24;
pub const STAGE60_BT_ELEMENT_COUNT: u32 = 10;

pub trait BtStage60Backend {
    fn read_gate_byte(&mut self) -> u8;
    fn read_selector_byte(&mut self) -> u8;
    fn read_source_byte(&mut self, address: u32) -> u8;
    fn read_target_byte(&mut self, address: u32) -> u8;
    fn write_target_byte(&mut self, address: u32, value: u8);
}

/// Safe source-level model of current `0x16F8D0`.
///
/// The source window intentionally starts at `base + selector * 10`; byte zero selects the
/// add/subtract mode and loop bytes one through ten provide adjustments. This means adjacent
/// selector windows overlap at the boundary exactly as the firmware arithmetic specifies.
pub fn bt_stage60_saturating_update<B: BtStage60Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    if backend.read_gate_byte() == 0 {
        return incoming_r0;
    }

    let selector = u32::from(backend.read_selector_byte());
    let source = STAGE60_BT_SOURCE_BASE_ADDR
        .wrapping_add(STAGE60_BT_SOURCE_WINDOW_STRIDE.wrapping_mul(selector));
    let subtract_mode = backend.read_source_byte(source) != 0;

    let mut index = 1u32;
    while index <= STAGE60_BT_ELEMENT_COUNT {
        let target = STAGE60_BT_TARGET_BASE_ADDR
            .wrapping_add(STAGE60_BT_TARGET_STRIDE.wrapping_mul(index - 1));
        let old = backend.read_target_byte(target);
        let delta = backend.read_source_byte(source.wrapping_add(index));
        let new = if subtract_mode {
            old.saturating_sub(delta)
        } else {
            old.saturating_add(delta)
        };
        backend.write_target_byte(target, new);
        index += 1;
    }

    source
}

#[cfg(test)]
mod stage60_tests {
    extern crate std;
    use super::*;
    use std::collections::BTreeMap;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        gate: u8,
        selector: u8,
        source: BTreeMap<u32, u8>,
        target: BTreeMap<u32, u8>,
        writes: Vec<(u32, u8)>,
    }

    impl BtStage60Backend for B {
        fn read_gate_byte(&mut self) -> u8 { self.gate }
        fn read_selector_byte(&mut self) -> u8 { self.selector }
        fn read_source_byte(&mut self, address: u32) -> u8 { *self.source.get(&address).unwrap_or(&0) }
        fn read_target_byte(&mut self, address: u32) -> u8 { *self.target.get(&address).unwrap_or(&0) }
        fn write_target_byte(&mut self, address: u32, value: u8) {
            self.target.insert(address, value);
            self.writes.push((address, value));
        }
    }

    fn target(i: u32) -> u32 {
        STAGE60_BT_TARGET_BASE_ADDR + STAGE60_BT_TARGET_STRIDE * i
    }

    #[test]
    fn zero_gate_returns_incoming_r0_without_target_writes() {
        let mut b = B::default();
        assert_eq!(bt_stage60_saturating_update(0xCAFE_BABE, &mut b), 0xCAFE_BABE);
        assert!(b.writes.is_empty());
    }

    #[test]
    fn zero_control_byte_saturating_adds_ten_elements() {
        let selector = 2u8;
        let source = STAGE60_BT_SOURCE_BASE_ADDR + u32::from(selector) * 10;
        let mut b = B { gate: 1, selector, ..Default::default() };
        b.source.insert(source, 0);
        for i in 0..10u32 {
            b.source.insert(source + i + 1, 20);
            b.target.insert(target(i), if i == 0 { 250 } else { i as u8 });
        }
        assert_eq!(bt_stage60_saturating_update(0, &mut b), source);
        assert_eq!(b.target[&target(0)], 255);
        assert_eq!(b.writes.len(), 10);
    }

    #[test]
    fn nonzero_control_byte_saturating_subtracts_with_floor_zero() {
        let selector = 1u8;
        let source = STAGE60_BT_SOURCE_BASE_ADDR + 10;
        let mut b = B { gate: 1, selector, ..Default::default() };
        b.source.insert(source, 1);
        for i in 0..10u32 {
            b.source.insert(source + i + 1, 10);
            b.target.insert(target(i), if i == 0 { 3 } else { 20 });
        }
        assert_eq!(bt_stage60_saturating_update(9, &mut b), source);
        assert_eq!(b.target[&target(0)], 0);
        assert_eq!(b.target[&target(1)], 10);
    }

    #[test]
    fn source_window_is_stride_ten_and_reads_offsets_one_through_ten() {
        let selector = 3u8;
        let source = STAGE60_BT_SOURCE_BASE_ADDR + 30;
        let mut b = B { gate: 1, selector, ..Default::default() };
        b.source.insert(source, 0);
        b.source.insert(source + 10, 7);
        for i in 0..10u32 { b.target.insert(target(i), 0); }
        let _ = bt_stage60_saturating_update(0, &mut b);
        assert_eq!(b.target[&target(9)], 7);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE60_CURRENT_BT_SATURATING_UPDATE_ADDR, 0x16F8D0);
        assert_eq!(STAGE60_BT_GATE_ADDR, 0x222747);
        assert_eq!(STAGE60_BT_SELECTOR_BASE_ADDR, 0x20DCD2);
        assert_eq!(STAGE60_BT_SELECTOR_OFFSET, 3);
        assert_eq!(STAGE60_BT_SOURCE_BASE_ADDR, 0x22270B);
        assert_eq!(STAGE60_BT_TARGET_BASE_ADDR, 0x20DE31);
        assert_eq!(STAGE60_BT_TARGET_STRIDE, 0x24);
        assert_eq!(STAGE60_BT_ELEMENT_COUNT, 10);
    }
}

/// Stage 61: current indirect dual-sample clamp at `0x16F928`.
///
/// This is a current-only reconstruction: the exact 60-byte body has one unique hit in the
/// current executable range, but no exact/prefix structural counterpart was found in the
/// available public legacy HCD. The two calls are indirect BLX operations through a runtime
/// function pointer; their broader contract remains opaque.
pub const STAGE61_CURRENT_BT_DUAL_SAMPLE_CLAMP_ADDR: u32 = 0x0016_F928;
pub const STAGE61_BT_GATE_ADDR: u32 = 0x0022_274A;
pub const STAGE61_BT_DISPATCH_PTR_ADDR: u32 = 0x0020_375C;
pub const STAGE61_BT_DISPATCH_METHOD_OFFSET: u32 = 0x5C;
pub const STAGE61_BT_OFFSET_BYTE_ADDR: u32 = 0x0020_D9A4;
pub const STAGE61_BT_SAMPLE_OFFSET: u32 = 0x15;

pub trait BtStage61Backend {
    /// Current ambient gate byte. Firmware reads it once before any indirect call and again
    /// after both calls, so backends may return different values on successive reads.
    fn read_gate_byte(&mut self) -> u8;

    /// Models a fresh load of `*0x20375C`, method dword `+0x5C`, and indirect BLX with the
    /// supplied selector. The returned value is the firmware R0 result of that call.
    fn invoke_selector(&mut self, selector: u32) -> u32;

    /// Current ambient offset byte `*0x20D9A4`, deliberately re-read after each call.
    fn read_offset_byte(&mut self) -> u8;

    fn read_byte(&mut self, address: u32) -> u8;
    fn write_byte(&mut self, address: u32, value: u8);
}

/// Safe source-level model of current `0x16F928`.
pub fn bt_stage61_dual_sample_clamp<B: BtStage61Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    if backend.read_gate_byte() == 0 {
        return incoming_r0;
    }

    let first_base = backend
        .invoke_selector(1)
        .wrapping_add(u32::from(backend.read_offset_byte()));
    let first_addr = first_base.wrapping_add(STAGE61_BT_SAMPLE_OFFSET);
    let first = backend.read_byte(first_addr);

    let second_base = backend
        .invoke_selector(0)
        .wrapping_add(u32::from(backend.read_offset_byte()));
    let post_gate = backend.read_gate_byte();
    let second_addr = second_base.wrapping_add(STAGE61_BT_SAMPLE_OFFSET);
    let second = backend.read_byte(second_addr);

    let difference = first.abs_diff(second);
    if post_gate < difference {
        backend.write_byte(second_addr, first);
    }

    second_base
}

#[cfg(test)]
mod stage61_tests {
    extern crate std;
    use super::*;
    use std::collections::BTreeMap;
    use std::vec;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        gates: Vec<u8>,
        offsets: Vec<u8>,
        returns: [u32; 2],
        memory: BTreeMap<u32, u8>,
        writes: Vec<(u32, u8)>,
        calls: Vec<(&'static str, u32)>,
    }

    impl BtStage61Backend for B {
        fn read_gate_byte(&mut self) -> u8 {
            self.calls.push(("gate", 0));
            self.gates.remove(0)
        }
        fn invoke_selector(&mut self, selector: u32) -> u32 {
            self.calls.push(("invoke", selector));
            self.returns[selector as usize]
        }
        fn read_offset_byte(&mut self) -> u8 {
            self.calls.push(("offset", 0));
            self.offsets.remove(0)
        }
        fn read_byte(&mut self, address: u32) -> u8 {
            self.calls.push(("read", address));
            *self.memory.get(&address).unwrap_or(&0)
        }
        fn write_byte(&mut self, address: u32, value: u8) {
            self.calls.push(("write", address));
            self.memory.insert(address, value);
            self.writes.push((address, value));
        }
    }

    #[test]
    fn zero_initial_gate_is_strict_early_exit() {
        let mut b = B { gates: vec![0], ..Default::default() };
        assert_eq!(bt_stage61_dual_sample_clamp(0xCAFE_BABE, &mut b), 0xCAFE_BABE);
        assert_eq!(b.calls, [("gate", 0)]);
    }

    #[test]
    fn offsets_and_gate_are_reread_after_indirect_calls() {
        let first_ret = 0x1000u32;
        let second_ret = 0x2000u32;
        let first_addr = first_ret + 3 + STAGE61_BT_SAMPLE_OFFSET;
        let second_addr = second_ret + 9 + STAGE61_BT_SAMPLE_OFFSET;
        let mut memory = BTreeMap::new();
        memory.insert(first_addr, 100);
        memory.insert(second_addr, 10);
        let mut b = B {
            gates: vec![1, 20],
            offsets: vec![3, 9],
            returns: [second_ret, first_ret],
            memory,
            ..Default::default()
        };
        assert_eq!(bt_stage61_dual_sample_clamp(0, &mut b), second_ret + 9);
        assert_eq!(b.writes, [(second_addr, 100)]);
        assert_eq!(b.calls.iter().filter(|x| x.0 == "offset").count(), 2);
        assert_eq!(b.calls.iter().filter(|x| x.0 == "gate").count(), 2);
    }

    #[test]
    fn threshold_is_strict_gate_less_than_absolute_difference() {
        let mut memory = BTreeMap::new();
        memory.insert(0x1000 + STAGE61_BT_SAMPLE_OFFSET, 90);
        memory.insert(0x2000 + STAGE61_BT_SAMPLE_OFFSET, 50);
        let mut b = B {
            gates: vec![1, 40],
            offsets: vec![0, 0],
            returns: [0x2000, 0x1000],
            memory,
            ..Default::default()
        };
        let _ = bt_stage61_dual_sample_clamp(0, &mut b);
        assert!(b.writes.is_empty());

        b.calls.clear(); b.gates = vec![1, 39]; b.offsets = vec![0, 0];
        let _ = bt_stage61_dual_sample_clamp(0, &mut b);
        assert_eq!(b.writes, [(0x2000 + STAGE61_BT_SAMPLE_OFFSET, 90)]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE61_CURRENT_BT_DUAL_SAMPLE_CLAMP_ADDR, 0x16F928);
        assert_eq!(STAGE61_BT_GATE_ADDR, 0x22274A);
        assert_eq!(STAGE61_BT_DISPATCH_PTR_ADDR, 0x20375C);
        assert_eq!(STAGE61_BT_DISPATCH_METHOD_OFFSET, 0x5C);
        assert_eq!(STAGE61_BT_OFFSET_BYTE_ADDR, 0x20D9A4);
        assert_eq!(STAGE61_BT_SAMPLE_OFFSET, 0x15);
    }
}

/// Stage 62: current three-selector record-byte initializer at `0x17022C`.
///
/// The exact current 70-byte leaf body is byte-identical to the public 73136-byte legacy
/// structural counterpart at `0x16CD34`. There are no runtime calls. The model preserves
/// the three-byte selector scan, 0xFF sentinel, fresh base-pointer load for each live
/// selector, exact 14-byte record stride, byte offset 11, literal value eight, and R0 shape.
pub const STAGE62_CURRENT_BT_SELECTOR_INIT_ADDR: u32 = 0x0017_022C;
pub const STAGE62_BT_SELECTOR_TABLE_ADDR: u32 = 0x0022_2750;
pub const STAGE62_BT_RECORD_BASE_PTR_ADDR: u32 = 0x0020_918C;
pub const STAGE62_BT_SELECTOR_COUNT: u32 = 3;
pub const STAGE62_BT_UNUSED_SELECTOR: u8 = 0xFF;
pub const STAGE62_BT_RECORD_STRIDE: u32 = 14;
pub const STAGE62_BT_RECORD_BYTE_OFFSET: u32 = 11;
pub const STAGE62_BT_INITIAL_VALUE: u8 = 8;

pub trait BtStage62Backend {
    fn read_selector_byte(&mut self, index: u32) -> u8;
    /// Firmware reloads `*0x20918C` independently for every non-0xFF selector.
    fn read_record_base(&mut self) -> u32;
    fn write_byte(&mut self, address: u32, value: u8);
}

/// Safe source-level model of current `0x17022C`.
pub fn bt_stage62_initialize_selected_records<B: BtStage62Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    let mut index = 0u32;
    while index < STAGE62_BT_SELECTOR_COUNT {
        let selector = backend.read_selector_byte(index);
        if selector != STAGE62_BT_UNUSED_SELECTOR {
            let base = backend.read_record_base();
            let record = base.wrapping_add(
                STAGE62_BT_RECORD_STRIDE.wrapping_mul(u32::from(selector)),
            );
            backend.write_byte(
                record.wrapping_add(STAGE62_BT_RECORD_BYTE_OFFSET),
                STAGE62_BT_INITIAL_VALUE,
            );
        }
        index += 1;
    }
    incoming_r0
}

#[cfg(test)]
mod stage62_tests {
    extern crate std;
    use super::*;
    use std::vec;
    use std::vec::Vec;

    struct B {
        selectors: [u8; 3],
        bases: Vec<u32>,
        base_reads: usize,
        writes: Vec<(u32, u8)>,
    }

    impl BtStage62Backend for B {
        fn read_selector_byte(&mut self, index: u32) -> u8 {
            self.selectors[index as usize]
        }
        fn read_record_base(&mut self) -> u32 {
            let value = self.bases[self.base_reads];
            self.base_reads += 1;
            value
        }
        fn write_byte(&mut self, address: u32, value: u8) {
            self.writes.push((address, value));
        }
    }

    #[test]
    fn all_ff_selectors_skip_base_reads_and_writes() {
        let mut b = B {
            selectors: [0xFF; 3],
            bases: Vec::new(),
            base_reads: 0,
            writes: Vec::new(),
        };
        assert_eq!(bt_stage62_initialize_selected_records(0xCAFE_BABE, &mut b), 0xCAFE_BABE);
        assert_eq!(b.base_reads, 0);
        assert!(b.writes.is_empty());
    }

    #[test]
    fn live_selectors_reload_base_and_use_stride_fourteen_offset_eleven() {
        let mut b = B {
            selectors: [1, 0xFF, 2],
            bases: vec![0x1000, 0x2000],
            base_reads: 0,
            writes: Vec::new(),
        };
        assert_eq!(bt_stage62_initialize_selected_records(7, &mut b), 7);
        assert_eq!(b.base_reads, 2);
        assert_eq!(b.writes, [
            (0x1000 + 14 + 11, 8),
            (0x2000 + 28 + 11, 8),
        ]);
    }

    #[test]
    fn selectors_are_processed_in_table_order() {
        let mut b = B {
            selectors: [2, 0, 1],
            bases: vec![0x100, 0x200, 0x300],
            base_reads: 0,
            writes: Vec::new(),
        };
        let _ = bt_stage62_initialize_selected_records(0, &mut b);
        assert_eq!(b.writes, [
            (0x100 + 28 + 11, 8),
            (0x200 + 11, 8),
            (0x300 + 14 + 11, 8),
        ]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE62_CURRENT_BT_SELECTOR_INIT_ADDR, 0x17022C);
        assert_eq!(STAGE62_BT_SELECTOR_TABLE_ADDR, 0x222750);
        assert_eq!(STAGE62_BT_RECORD_BASE_PTR_ADDR, 0x20918C);
        assert_eq!(STAGE62_BT_SELECTOR_COUNT, 3);
        assert_eq!(STAGE62_BT_UNUSED_SELECTOR, 0xFF);
        assert_eq!(STAGE62_BT_RECORD_STRIDE, 14);
        assert_eq!(STAGE62_BT_RECORD_BYTE_OFFSET, 11);
        assert_eq!(STAGE62_BT_INITIAL_VALUE, 8);
    }
}

/// Stage 63: current optional-boundary sequence at `0x1704D4`.
///
/// After masking only the three direct BL encodings, the current 34-byte body has one
/// structural hit in current firmware and one in public legacy (`0x16CFDC`). The wider
/// runtime meaning of all three callees remains opaque. This model preserves the single
/// context load, call arguments/order, current-R0 forwarding to the optional final call,
/// saved-input gate, and unconditional zero return.
pub const STAGE63_CURRENT_BT_OPTIONAL_SEQUENCE_ADDR: u32 = 0x0017_04D4;
pub const STAGE63_BT_CONTEXT_PTR_ADDR: u32 = 0x0022_257C;
pub const STAGE63_BT_FIRST_BOUNDARY: u32 = 0x0016_3668;
pub const STAGE63_BT_SECOND_BOUNDARY: u32 = 0x0000_3D24;
pub const STAGE63_BT_FINAL_BOUNDARY: u32 = 0x0017_1A7C;

pub trait BtStage63Backend {
    fn read_context_ptr(&mut self) -> u32;
    fn first_boundary(&mut self, incoming_r0: u32) -> u32;
    fn second_boundary(&mut self, context: u32, zero: u32, first_result: u32) -> u32;
    fn final_boundary(&mut self, current_r0: u32) -> u32;
}

/// Safe source-level model of current `0x1704D4`.
pub fn bt_stage63_optional_sequence<B: BtStage63Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    let context = backend.read_context_ptr();
    let saved_input = incoming_r0;
    let mut current_r0 = incoming_r0;

    if context != 0 {
        current_r0 = backend.first_boundary(current_r0);
        current_r0 = backend.second_boundary(context, 0, current_r0);
    }

    if saved_input != 0 {
        let _ = backend.final_boundary(current_r0);
    }

    0
}

#[cfg(test)]
mod stage63_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        context: u32,
        first_result: u32,
        second_result: u32,
        final_result: u32,
        calls: Vec<(&'static str, u32, u32, u32)>,
    }

    impl BtStage63Backend for B {
        fn read_context_ptr(&mut self) -> u32 {
            self.calls.push(("context", 0, 0, 0));
            self.context
        }
        fn first_boundary(&mut self, incoming_r0: u32) -> u32 {
            self.calls.push(("first", incoming_r0, 0, 0));
            self.first_result
        }
        fn second_boundary(&mut self, context: u32, zero: u32, first_result: u32) -> u32 {
            self.calls.push(("second", context, zero, first_result));
            self.second_result
        }
        fn final_boundary(&mut self, current_r0: u32) -> u32 {
            self.calls.push(("final", current_r0, 0, 0));
            self.final_result
        }
    }

    #[test]
    fn zero_context_and_zero_input_only_read_context() {
        let mut b = B::default();
        assert_eq!(bt_stage63_optional_sequence(0, &mut b), 0);
        assert_eq!(b.calls, [("context", 0, 0, 0)]);
    }

    #[test]
    fn zero_context_nonzero_input_forwards_original_r0_to_final() {
        let mut b = B { final_result: 0xAAAA, ..Default::default() };
        assert_eq!(bt_stage63_optional_sequence(0x1234, &mut b), 0);
        assert_eq!(b.calls, [
            ("context", 0, 0, 0),
            ("final", 0x1234, 0, 0),
        ]);
    }

    #[test]
    fn nonzero_context_zero_input_runs_first_two_but_skips_final() {
        let mut b = B {
            context: 0x9000,
            first_result: 0x1111,
            second_result: 0x2222,
            ..Default::default()
        };
        assert_eq!(bt_stage63_optional_sequence(0, &mut b), 0);
        assert_eq!(b.calls, [
            ("context", 0, 0, 0),
            ("first", 0, 0, 0),
            ("second", 0x9000, 0, 0x1111),
        ]);
    }

    #[test]
    fn nonzero_context_nonzero_input_forwards_second_return_to_final() {
        let mut b = B {
            context: 0x9000,
            first_result: 0x1111,
            second_result: 0x2222,
            final_result: 0x3333,
            ..Default::default()
        };
        assert_eq!(bt_stage63_optional_sequence(0xABCD, &mut b), 0);
        assert_eq!(b.calls, [
            ("context", 0, 0, 0),
            ("first", 0xABCD, 0, 0),
            ("second", 0x9000, 0, 0x1111),
            ("final", 0x2222, 0, 0),
        ]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE63_CURRENT_BT_OPTIONAL_SEQUENCE_ADDR, 0x1704D4);
        assert_eq!(STAGE63_BT_CONTEXT_PTR_ADDR, 0x22257C);
        assert_eq!(STAGE63_BT_FIRST_BOUNDARY, 0x163668);
        assert_eq!(STAGE63_BT_SECOND_BOUNDARY, 0x3D24);
        assert_eq!(STAGE63_BT_FINAL_BOUNDARY, 0x171A7C);
    }
}

/// Stage 64: current first-exact-one index scan at `0x1720C0`.
///
/// The exact current 24-byte wrapper contains one direct call. Masking only that BL encoding
/// leaves 20 fixed bytes and yields one current structural hit plus one public-legacy
/// structural counterpart at `0x16E010`. The callee contract remains opaque; this model
/// preserves the exact index sequence, equality-to-one stop condition, and sentinel return.
pub const STAGE64_CURRENT_BT_EXACT_ONE_SCAN_ADDR: u32 = 0x0017_20C0;
pub const STAGE64_BT_PREDICATE_BOUNDARY: u32 = 0x0017_2044;
pub const STAGE64_BT_SCAN_COUNT: u32 = 8;
pub const STAGE64_BT_MATCH_VALUE: u32 = 1;

pub trait BtStage64Backend {
    /// Current `0x172044(index)`. Firmware passes indices zero through seven as zero-extended
    /// bytes and stops only when this opaque boundary returns exactly one.
    fn predicate_boundary(&mut self, index: u8) -> u32;
}

/// Safe source-level model of current `0x1720C0`.
///
/// Incoming R0 is overwritten before the first call. The return is the first matching index,
/// or literal eight when no call returns exactly one.
pub fn bt_stage64_first_exact_one<B: BtStage64Backend>(
    _incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    let mut index = 0u32;
    while index < STAGE64_BT_SCAN_COUNT {
        if backend.predicate_boundary(index as u8) == STAGE64_BT_MATCH_VALUE {
            return index;
        }
        index += 1;
    }
    STAGE64_BT_SCAN_COUNT
}

#[cfg(test)]
mod stage64_tests {
    use super::*;

    struct B {
        returns: [u32; 8],
        calls: [u8; 8],
        count: usize,
    }

    impl B {
        fn new(returns: [u32; 8]) -> Self {
            Self { returns, calls: [0; 8], count: 0 }
        }
    }

    impl BtStage64Backend for B {
        fn predicate_boundary(&mut self, index: u8) -> u32 {
            self.calls[self.count] = index;
            self.count += 1;
            self.returns[index as usize]
        }
    }

    #[test]
    fn first_exact_one_stops_immediately() {
        let mut b = B::new([1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(bt_stage64_first_exact_one(0xDEAD_BEEF, &mut b), 0);
        assert_eq!(b.count, 1);
        assert_eq!(&b.calls[..b.count], &[0]);
    }

    #[test]
    fn non_one_values_do_not_match() {
        let mut b = B::new([0, 2, u32::MAX, 3, 1, 1, 1, 1]);
        assert_eq!(bt_stage64_first_exact_one(7, &mut b), 4);
        assert_eq!(b.count, 5);
        assert_eq!(&b.calls[..b.count], &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn no_match_returns_eight_after_all_indices() {
        let mut b = B::new([0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bt_stage64_first_exact_one(123, &mut b), 8);
        assert_eq!(b.count, 8);
        assert_eq!(b.calls, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE64_CURRENT_BT_EXACT_ONE_SCAN_ADDR, 0x1720C0);
        assert_eq!(STAGE64_BT_PREDICATE_BOUNDARY, 0x172044);
        assert_eq!(STAGE64_BT_SCAN_COUNT, 8);
        assert_eq!(STAGE64_BT_MATCH_VALUE, 1);
    }
}

/// Stage 65: current zero-fallback mode wrapper at `0x1724C8`.
///
/// The exact current 28-byte body contains one ordinary BL and one final B.W to the same
/// opaque boundary. Masking both four-byte control-transfer encodings leaves 20 fixed bytes
/// and yields one current structural hit plus one public-legacy counterpart at `0x16E3B8`.
/// This model preserves the mode-zero probe, exact nonzero gate, mode-one fallback, and
/// tail-result forwarding without assigning wider meaning to the boundary.
pub const STAGE65_CURRENT_BT_ZERO_FALLBACK_ADDR: u32 = 0x0017_24C8;
pub const STAGE65_BT_BOUNDARY: u32 = 0x0017_2458;
pub const STAGE65_BT_PRIMARY_MODE: u32 = 0;
pub const STAGE65_BT_FALLBACK_MODE: u32 = 1;

pub trait BtStage65Backend {
    /// Current `0x172458(mode, input)`.
    fn boundary(&mut self, mode: u32, input: u32) -> u32;
}

/// Safe source-level model of current `0x1724C8`.
///
/// Firmware calls mode zero first. Any nonzero return is final. Only exact zero triggers a
/// frame restore followed by a tail branch to the same boundary with mode one.
pub fn bt_stage65_zero_fallback<B: BtStage65Backend>(
    input: u32,
    backend: &mut B,
) -> u32 {
    let first = backend.boundary(STAGE65_BT_PRIMARY_MODE, input);
    if first != 0 {
        return first;
    }
    backend.boundary(STAGE65_BT_FALLBACK_MODE, input)
}

#[cfg(test)]
mod stage65_tests {
    use super::*;

    #[derive(Default)]
    struct B {
        primary: u32,
        fallback: u32,
        calls: [(u32, u32); 2],
        count: usize,
    }

    impl BtStage65Backend for B {
        fn boundary(&mut self, mode: u32, input: u32) -> u32 {
            self.calls[self.count] = (mode, input);
            self.count += 1;
            if mode == 0 { self.primary } else { self.fallback }
        }
    }

    #[test]
    fn nonzero_primary_return_skips_fallback_and_is_final() {
        let mut b = B { primary: 7, fallback: 99, ..Default::default() };
        assert_eq!(bt_stage65_zero_fallback(0x1234, &mut b), 7);
        assert_eq!(b.count, 1);
        assert_eq!(b.calls[0], (0, 0x1234));
    }

    #[test]
    fn exact_zero_primary_tail_forwards_mode_one_result() {
        let mut b = B { primary: 0, fallback: 0xDEAD_BEEF, ..Default::default() };
        assert_eq!(bt_stage65_zero_fallback(0xABCD, &mut b), 0xDEAD_BEEF);
        assert_eq!(b.count, 2);
        assert_eq!(b.calls, [(0, 0xABCD), (1, 0xABCD)]);
    }

    #[test]
    fn fallback_zero_is_preserved() {
        let mut b = B::default();
        assert_eq!(bt_stage65_zero_fallback(0, &mut b), 0);
        assert_eq!(b.calls, [(0, 0), (1, 0)]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE65_CURRENT_BT_ZERO_FALLBACK_ADDR, 0x1724C8);
        assert_eq!(STAGE65_BT_BOUNDARY, 0x172458);
        assert_eq!(STAGE65_BT_PRIMARY_MODE, 0);
        assert_eq!(STAGE65_BT_FALLBACK_MODE, 1);
    }
}

/// Stage 66: current bounded tail-copy wrapper at `0x1724E4`.
///
/// The exact current 46-byte body contains one ordinary BL and one final B.W. Masking
/// those two four-byte control-transfer encodings leaves 38 fixed bytes and yields one
/// current structural hit plus one public-legacy counterpart at `0x16E3D4`.
/// This model preserves the first opaque boundary, the intervening header-byte store,
/// unsigned length clamp, source preservation, and final tail-result forwarding.
pub const STAGE66_CURRENT_BT_BOUNDED_TAIL_COPY_ADDR: u32 = 0x0017_24E4;
pub const STAGE66_BT_FIRST_BOUNDARY: u32 = 0x0000_3D24;
pub const STAGE66_BT_TAIL_BOUNDARY: u32 = 0x0000_3DB4;
pub const STAGE66_BT_CURRENT_BUFFER_BASE: u32 = 0x0022_300E;
pub const STAGE66_BT_CLEAR_LENGTH: u32 = 0x3B;
pub const STAGE66_BT_INLINE_LIMIT: u32 = 0x39;
pub const STAGE66_BT_COPY_LIMIT: u32 = 0x3A;

pub trait BtStage66Backend {
    /// Current `0x3D24(buffer, 0, 59)`. The return is ignored locally.
    fn first_boundary(&mut self, buffer: u32, value: u32, len: u32) -> u32;

    /// Exact local byte store performed after the first boundary returns.
    fn write_header_byte(&mut self, address: u32, value: u8);

    /// Current tail boundary `0x3DB4(buffer + 1, source, len)`.
    fn tail_boundary(&mut self, destination: u32, source: u32, len: u32) -> u32;
}

/// Safe source-level model of current `0x1724E4`.
///
/// Firmware first invokes the opaque `0x3D24` boundary with `(base, 0, 59)`. It then
/// stores the low byte of a logical right shift by one into `base`, preserves the incoming
/// source value, clamps the unsigned input length to at most 58, restores its frame, and
/// tail-branches to `0x3DB4(base + 1, source, clamped_len)`. The tail return is final.
pub fn bt_stage66_bounded_tail_copy<B: BtStage66Backend>(
    input_len: u32,
    source: u32,
    backend: &mut B,
) -> u32 {
    let base = STAGE66_BT_CURRENT_BUFFER_BASE;
    let _ = backend.first_boundary(base, 0, STAGE66_BT_CLEAR_LENGTH);

    backend.write_header_byte(base, (input_len >> 1) as u8);

    let copy_len = if input_len <= STAGE66_BT_INLINE_LIMIT {
        input_len
    } else {
        STAGE66_BT_COPY_LIMIT
    };

    backend.tail_boundary(base.wrapping_add(1), source, copy_len)
}

#[cfg(test)]
mod stage66_tests {
    use super::*;

    #[derive(Default)]
    struct B {
        first_args: (u32, u32, u32),
        first_return: u32,
        header: (u32, u8),
        tail_args: (u32, u32, u32),
        tail_return: u32,
        first_step: u8,
        header_step: u8,
        tail_step: u8,
        next_step: u8,
    }

    impl B {
        fn step(&mut self) -> u8 {
            self.next_step += 1;
            self.next_step
        }
    }

    impl BtStage66Backend for B {
        fn first_boundary(&mut self, buffer: u32, value: u32, len: u32) -> u32 {
            let step = self.step();
            self.first_step = step;
            self.first_args = (buffer, value, len);
            self.first_return
        }

        fn write_header_byte(&mut self, address: u32, value: u8) {
            let step = self.step();
            self.header_step = step;
            self.header = (address, value);
        }

        fn tail_boundary(&mut self, destination: u32, source: u32, len: u32) -> u32 {
            let step = self.step();
            self.tail_step = step;
            self.tail_args = (destination, source, len);
            self.tail_return
        }
    }

    #[test]
    fn first_boundary_header_store_and_tail_are_ordered() {
        let mut b = B { first_return: 0xAAAA_AAAA, tail_return: 0xDEAD_BEEF, ..Default::default() };
        assert_eq!(bt_stage66_bounded_tail_copy(5, 0x1234_5678, &mut b), 0xDEAD_BEEF);
        assert_eq!(b.first_args, (STAGE66_BT_CURRENT_BUFFER_BASE, 0, 59));
        assert_eq!(b.header, (STAGE66_BT_CURRENT_BUFFER_BASE, 2));
        assert_eq!(b.tail_args, (STAGE66_BT_CURRENT_BUFFER_BASE + 1, 0x1234_5678, 5));
        assert_eq!((b.first_step, b.header_step, b.tail_step), (1, 2, 3));
    }

    #[test]
    fn unsigned_length_clamp_changes_only_values_above_57() {
        let mut b = B { tail_return: 1, ..Default::default() };
        let _ = bt_stage66_bounded_tail_copy(57, 9, &mut b);
        assert_eq!(b.tail_args.2, 57);
        let _ = bt_stage66_bounded_tail_copy(58, 9, &mut b);
        assert_eq!(b.tail_args.2, 58);
        let _ = bt_stage66_bounded_tail_copy(u32::MAX, 9, &mut b);
        assert_eq!(b.tail_args.2, 58);
    }

    #[test]
    fn header_is_low_byte_of_logical_shift_and_first_return_is_ignored() {
        let mut b = B { first_return: 7, tail_return: 11, ..Default::default() };
        assert_eq!(bt_stage66_bounded_tail_copy(u32::MAX, 3, &mut b), 11);
        assert_eq!(b.header.1, 0xFF);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE66_CURRENT_BT_BOUNDED_TAIL_COPY_ADDR, 0x1724E4);
        assert_eq!(STAGE66_BT_FIRST_BOUNDARY, 0x3D24);
        assert_eq!(STAGE66_BT_TAIL_BOUNDARY, 0x3DB4);
        assert_eq!(STAGE66_BT_CURRENT_BUFFER_BASE, 0x22300E);
        assert_eq!(STAGE66_BT_CLEAR_LENGTH, 59);
        assert_eq!(STAGE66_BT_INLINE_LIMIT, 57);
        assert_eq!(STAGE66_BT_COPY_LIMIT, 58);
    }
}

/// Stage 67: current five-way state dispatch wrapper at `0x172518`.
///
/// The exact current 98-byte body contains a TBB dispatch and four direct calls. Masking
/// only those four four-byte call encodings leaves 82 fixed bytes and yields one current
/// structural hit plus one public-legacy counterpart at `0x16E408`.
pub const STAGE67_CURRENT_BT_STATE_DISPATCH_ADDR: u32 = 0x0017_2518;
pub const STAGE67_BT_STATE_ADDR: u32 = 0x0022_3064;
pub const STAGE67_BT_SOURCE_BYTE_ADDR: u32 = 0x0022_2084;
pub const STAGE67_BT_CONTEXT_WORD_ADDR: u32 = 0x0022_208C;
pub const STAGE67_BT_DEST_BYTE_ADDR: u32 = 0x0022_3065;
pub const STAGE67_BT_CALLBACK_PTR: u32 = 0x0017_1FF9;
pub const STAGE67_BT_BLOCK_ADDR: u32 = 0x0022_304C;
pub const STAGE67_BT_ZERO_BOUNDARY: u32 = 0x0007_2B24;
pub const STAGE67_BT_REGISTER_BOUNDARY: u32 = 0x0001_51FE;
pub const STAGE67_BT_ALTERNATE_BOUNDARY: u32 = 0x0001_51BC;
pub const STAGE67_BT_FINALIZE_BOUNDARY: u32 = 0x0001_5180;

pub trait BtStage67Backend {
    fn read_state_byte(&mut self) -> u8;
    fn write_state_byte(&mut self, value: u8);
    fn read_source_byte(&mut self) -> u8;
    fn write_dest_byte(&mut self, value: u8);
    fn read_context_word(&mut self) -> u32;

    /// Current `0x72B24(0, incoming_r1, incoming_r2, 0)` on state zero.
    fn zero_boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;
    /// Current `0x151FE(block, callback, 0, context)` on state zero.
    fn register_boundary(&mut self, block: u32, callback: u32, zero: u32, context: u32) -> u32;
    /// Current `0x151BC(block, incoming_r1, copied_byte, dest_addr)` on state one.
    fn alternate_boundary(&mut self, block: u32, incoming_r1: u32, value: u32, dest_addr: u32) -> u32;
    /// Current `0x15180(block, context)` on states zero and one.
    fn finalize_boundary(&mut self, block: u32, context: u32) -> u32;
}

/// Safe source-level model of current `0x172518`.
///
/// Incoming R0 is not consumed by the visible body. State zero runs the zero/init route;
/// state one runs the alternate route; states two through four are rewritten to five;
/// states above four skip dispatch work. The final state byte is always re-read and maps
/// to return value three when it is at least two, otherwise zero.
pub fn bt_stage67_state_dispatch<B: BtStage67Backend>(
    _incoming_r0: u32,
    incoming_r1: u32,
    incoming_r2: u32,
    backend: &mut B,
) -> u32 {
    let state = backend.read_state_byte();
    match state {
        0 => {
            let _ = backend.zero_boundary(0, incoming_r1, incoming_r2, 0);
            backend.write_state_byte(1);

            let value = backend.read_source_byte();
            backend.write_dest_byte(value);

            let context = backend.read_context_word();
            let _ = backend.register_boundary(
                STAGE67_BT_BLOCK_ADDR,
                STAGE67_BT_CALLBACK_PTR,
                0,
                context,
            );

            let context = backend.read_context_word();
            let _ = backend.finalize_boundary(STAGE67_BT_BLOCK_ADDR, context);
        }
        1 => {
            let value = backend.read_source_byte();
            backend.write_dest_byte(value);
            let _ = backend.alternate_boundary(
                STAGE67_BT_BLOCK_ADDR,
                incoming_r1,
                u32::from(value),
                STAGE67_BT_DEST_BYTE_ADDR,
            );

            let context = backend.read_context_word();
            let _ = backend.finalize_boundary(STAGE67_BT_BLOCK_ADDR, context);
        }
        2..=4 => backend.write_state_byte(5),
        _ => {}
    }

    if backend.read_state_byte() >= 2 { 3 } else { 0 }
}

#[cfg(test)]
mod stage67_tests {
    use super::*;

    #[derive(Default)]
    struct B {
        state: u8,
        source: u8,
        context: u32,
        next_context_after_register: Option<u32>,
        state_after_finalize: Option<u8>,
        events: [u8; 16],
        count: usize,
        zero_args: (u32, u32, u32, u32),
        register_args: (u32, u32, u32, u32),
        alternate_args: (u32, u32, u32, u32),
        finalize_args: (u32, u32),
    }

    impl B {
        fn event(&mut self, id: u8) {
            self.events[self.count] = id;
            self.count += 1;
        }
    }

    impl BtStage67Backend for B {
        fn read_state_byte(&mut self) -> u8 { self.event(1); self.state }
        fn write_state_byte(&mut self, value: u8) { self.event(2); self.state = value; }
        fn read_source_byte(&mut self) -> u8 { self.event(3); self.source }
        fn write_dest_byte(&mut self, _value: u8) { self.event(4); }
        fn read_context_word(&mut self) -> u32 { self.event(5); self.context }
        fn zero_boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32 {
            self.event(6); self.zero_args = (r0, r1, r2, r3); 0xAAAA_AAAA
        }
        fn register_boundary(&mut self, block: u32, callback: u32, zero: u32, context: u32) -> u32 {
            self.event(7); self.register_args = (block, callback, zero, context);
            if let Some(v) = self.next_context_after_register { self.context = v; }
            0xBBBB_BBBB
        }
        fn alternate_boundary(&mut self, block: u32, incoming_r1: u32, value: u32, dest_addr: u32) -> u32 {
            self.event(8); self.alternate_args = (block, incoming_r1, value, dest_addr); 0xCCCC_CCCC
        }
        fn finalize_boundary(&mut self, block: u32, context: u32) -> u32 {
            self.event(9); self.finalize_args = (block, context);
            if let Some(v) = self.state_after_finalize { self.state = v; }
            0xDDDD_DDDD
        }
    }

    #[test]
    fn state_zero_preserves_call_order_and_rereads_context_and_final_state() {
        let mut b = B {
            state: 0,
            source: 0x5A,
            context: 0x1111,
            next_context_after_register: Some(0x2222),
            state_after_finalize: Some(2),
            ..Default::default()
        };
        assert_eq!(bt_stage67_state_dispatch(99, 0x12, 0x34, &mut b), 3);
        assert_eq!(b.zero_args, (0, 0x12, 0x34, 0));
        assert_eq!(b.register_args, (STAGE67_BT_BLOCK_ADDR, STAGE67_BT_CALLBACK_PTR, 0, 0x1111));
        assert_eq!(b.finalize_args, (STAGE67_BT_BLOCK_ADDR, 0x2222));
        assert_eq!(&b.events[..b.count], &[1,6,2,3,4,5,7,5,9,1]);
    }

    #[test]
    fn state_one_forwards_incoming_r1_and_copied_byte_then_uses_post_call_state() {
        let mut b = B { state: 1, source: 7, context: 0x3333, state_after_finalize: Some(0), ..Default::default() };
        assert_eq!(bt_stage67_state_dispatch(88, 0xABCD, 0xEEEE, &mut b), 0);
        assert_eq!(b.alternate_args, (STAGE67_BT_BLOCK_ADDR, 0xABCD, 7, STAGE67_BT_DEST_BYTE_ADDR));
        assert_eq!(b.finalize_args, (STAGE67_BT_BLOCK_ADDR, 0x3333));
        assert_eq!(&b.events[..b.count], &[1,3,4,8,5,9,1]);
    }

    #[test]
    fn states_two_through_four_are_rewritten_to_five() {
        for initial in 2..=4 {
            let mut b = B { state: initial, ..Default::default() };
            assert_eq!(bt_stage67_state_dispatch(0, 0, 0, &mut b), 3);
            assert_eq!(b.state, 5);
            assert_eq!(&b.events[..b.count], &[1,2,1]);
        }
    }

    #[test]
    fn states_above_four_only_reread_for_final_mapping() {
        let mut b = B { state: 0xFF, ..Default::default() };
        assert_eq!(bt_stage67_state_dispatch(1, 2, 3, &mut b), 3);
        assert_eq!(&b.events[..b.count], &[1,1]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE67_CURRENT_BT_STATE_DISPATCH_ADDR, 0x172518);
        assert_eq!(STAGE67_BT_STATE_ADDR, 0x223064);
        assert_eq!(STAGE67_BT_SOURCE_BYTE_ADDR, 0x222084);
        assert_eq!(STAGE67_BT_CONTEXT_WORD_ADDR, 0x22208C);
        assert_eq!(STAGE67_BT_DEST_BYTE_ADDR, 0x223065);
        assert_eq!(STAGE67_BT_CALLBACK_PTR, 0x171FF9);
        assert_eq!(STAGE67_BT_BLOCK_ADDR, 0x22304C);
        assert_eq!(STAGE67_BT_ZERO_BOUNDARY, 0x72B24);
        assert_eq!(STAGE67_BT_REGISTER_BOUNDARY, 0x151FE);
        assert_eq!(STAGE67_BT_ALTERNATE_BOUNDARY, 0x151BC);
        assert_eq!(STAGE67_BT_FINALIZE_BOUNDARY, 0x15180);
    }
}

/// Stage 68: current conditional context-registration leaf at `0x1726D0`.
///
/// This is a current-HCD-first reconstruction. The 40-byte current body has one
/// unique normalized hit in the current executable range. No public-legacy
/// structural counterpart is promoted for this stage.
pub const STAGE68_CURRENT_BT_CONDITIONAL_CONTEXT_REGISTER_ADDR: u32 = 0x0017_26D0;
pub const STAGE68_BT_CONTEXT_ADDR: u32 = 0x0022_3088;
pub const STAGE68_BT_CONTEXT_FLAG_WORD_ADDR: u32 = 0x0022_3090;
pub const STAGE68_BT_CONTEXT_FLAG_MASK: u32 = 0x0000_0004;
pub const STAGE68_BT_CALLBACK_THUMB: u32 = 0x0017_2661;
pub const STAGE68_BT_RESET_BYTE_ADDR: u32 = 0x0022_3080;
pub const STAGE68_BT_REGISTER_BOUNDARY: u32 = 0x0001_51FE;
pub const STAGE68_BT_FINALIZE_BOUNDARY: u32 = 0x0001_5180;

pub trait BtStage68Backend {
    /// Reads current dword `[0x223088 + 8]` before any runtime boundary.
    fn read_context_flag_word(&mut self) -> u32;

    /// Current `0x151FE(context, callback_thumb, 0, argument)`.
    /// Its return value is ignored locally.
    fn register_boundary(
        &mut self,
        context: u32,
        callback_thumb: u32,
        zero: u32,
        argument: u32,
    ) -> u32;

    /// Current `0x15180(context, argument)`. Its return survives the final byte
    /// store and is the function return on the active path.
    fn finalize_boundary(&mut self, context: u32, argument: u32) -> u32;

    /// Exact final byte write to current `0x223080`.
    fn write_reset_byte(&mut self, value: u8);
}

/// Safe source-level model of current `0x1726D0`.
///
/// When context flag bit 2 is already set, the firmware returns the incoming
/// argument unchanged and performs no other visible work. Otherwise it invokes
/// the register and finalize boundaries in order, writes zero to the reset byte,
/// and returns the finalize-boundary result.
pub fn bt_stage68_conditional_context_register<B: BtStage68Backend>(
    argument: u32,
    backend: &mut B,
) -> u32 {
    if backend.read_context_flag_word() & STAGE68_BT_CONTEXT_FLAG_MASK != 0 {
        return argument;
    }

    let _ = backend.register_boundary(
        STAGE68_BT_CONTEXT_ADDR,
        STAGE68_BT_CALLBACK_THUMB,
        0,
        argument,
    );
    let result = backend.finalize_boundary(STAGE68_BT_CONTEXT_ADDR, argument);
    backend.write_reset_byte(0);
    result
}

#[cfg(test)]
mod stage68_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        flags: u32,
        register_return: u32,
        finalize_return: u32,
        events: Vec<&'static str>,
        register_args: (u32, u32, u32, u32),
        finalize_args: (u32, u32),
        reset_value: Option<u8>,
    }

    impl BtStage68Backend for B {
        fn read_context_flag_word(&mut self) -> u32 {
            self.events.push("flags");
            self.flags
        }

        fn register_boundary(
            &mut self,
            context: u32,
            callback_thumb: u32,
            zero: u32,
            argument: u32,
        ) -> u32 {
            self.events.push("register");
            self.register_args = (context, callback_thumb, zero, argument);
            self.register_return
        }

        fn finalize_boundary(&mut self, context: u32, argument: u32) -> u32 {
            self.events.push("finalize");
            self.finalize_args = (context, argument);
            self.finalize_return
        }

        fn write_reset_byte(&mut self, value: u8) {
            self.events.push("reset");
            self.reset_value = Some(value);
        }
    }

    #[test]
    fn bit2_set_is_strict_early_return_of_incoming_argument() {
        let mut b = B { flags: STAGE68_BT_CONTEXT_FLAG_MASK, finalize_return: 0xDEAD_BEEF, ..Default::default() };
        assert_eq!(bt_stage68_conditional_context_register(0x1234_5678, &mut b), 0x1234_5678);
        assert_eq!(b.events, ["flags"]);
        assert_eq!(b.reset_value, None);
    }

    #[test]
    fn active_path_preserves_exact_boundary_arguments_and_order() {
        let mut b = B { register_return: 0xAAAA_AAAA, finalize_return: 0xBBBB_BBBB, ..Default::default() };
        assert_eq!(bt_stage68_conditional_context_register(0x1357_2468, &mut b), 0xBBBB_BBBB);
        assert_eq!(b.register_args, (STAGE68_BT_CONTEXT_ADDR, STAGE68_BT_CALLBACK_THUMB, 0, 0x1357_2468));
        assert_eq!(b.finalize_args, (STAGE68_BT_CONTEXT_ADDR, 0x1357_2468));
        assert_eq!(b.events, ["flags", "register", "finalize", "reset"]);
    }

    #[test]
    fn register_return_is_ignored_finalize_return_survives_reset_store() {
        let mut b = B { register_return: 7, finalize_return: 11, ..Default::default() };
        assert_eq!(bt_stage68_conditional_context_register(3, &mut b), 11);
        assert_eq!(b.reset_value, Some(0));
    }

    #[test]
    fn unrelated_flag_bits_do_not_block_active_path() {
        let mut b = B { flags: 0xFFFF_FFFB, finalize_return: 9, ..Default::default() };
        assert_eq!(bt_stage68_conditional_context_register(5, &mut b), 9);
        assert_eq!(b.events, ["flags", "register", "finalize", "reset"]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE68_CURRENT_BT_CONDITIONAL_CONTEXT_REGISTER_ADDR, 0x1726D0);
        assert_eq!(STAGE68_BT_CONTEXT_ADDR, 0x223088);
        assert_eq!(STAGE68_BT_CONTEXT_FLAG_WORD_ADDR, 0x223090);
        assert_eq!(STAGE68_BT_CONTEXT_FLAG_MASK, 4);
        assert_eq!(STAGE68_BT_CALLBACK_THUMB, 0x172661);
        assert_eq!(STAGE68_BT_RESET_BYTE_ADDR, 0x223080);
        assert_eq!(STAGE68_BT_REGISTER_BOUNDARY, 0x151FE);
        assert_eq!(STAGE68_BT_FINALIZE_BOUNDARY, 0x15180);
    }
}

/// Stage 69: current bounded callback/counter leaf at `0x172660`.
///
/// This callback is registered by Stage 68 through raw Thumb pointer `0x172661`.
/// No public-legacy structural counterpart is promoted for this current-only body.
pub const STAGE69_CURRENT_BT_BOUNDED_CALLBACK_ADDR: u32 = 0x0017_2660;
pub const STAGE69_BT_SOURCE_BYTE_ADDR: u32 = 0x0020_2FD4;
pub const STAGE69_BT_COUNTER_ADDR: u32 = 0x0022_3080;
pub const STAGE69_BT_LIMIT_ADDR: u32 = 0x0022_3084;
pub const STAGE69_BT_CONTROL_BYTE_ADDR: u32 = 0x0020_6800;
pub const STAGE69_BT_CONTEXT_ADDR: u32 = 0x0022_3088;
pub const STAGE69_BT_PROBE_BOUNDARY: u32 = 0x000B_AC7C;
pub const STAGE69_BT_NOTIFY_BOUNDARY: u32 = 0x0007_2D00;
pub const STAGE69_BT_CONTEXT_BOUNDARY: u32 = 0x0001_51BC;
pub const STAGE69_BT_CLEAR_BOUNDARY: u32 = 0x0000_3D24;
pub const STAGE69_BT_CLEAR_BYTES: u32 = 24;

pub trait BtStage69Backend {
    fn read_source_byte(&mut self) -> u8;
    /// Current `0xBAC7C(source)`. Only the low returned byte is used locally.
    fn probe_boundary(&mut self, source: u32) -> u32;
    fn read_counter_byte(&mut self) -> u8;
    fn read_limit_byte(&mut self) -> u8;
    fn write_counter_byte(&mut self, value: u8);
    fn read_control_byte(&mut self) -> u8;
    /// Current `0x72D00(value)`; return is final only on the fast path.
    fn notify_boundary(&mut self, value: u32) -> u32;
    /// Current `0x151BC(context)`; return is ignored locally.
    fn context_boundary(&mut self, context: u32) -> u32;
    /// Current tail `0x3D24(context, 0, 24)` on the overflow path.
    fn clear_boundary(&mut self, context: u32, value: u32, len: u32) -> u32;
}

/// Safe source-level model of current `0x172660`.
///
/// Firmware writes the wrapped incremented counter before deciding the path.
/// If `limit >= next`, it tail-notifies whether the low probe byte is zero.
/// Otherwise it resets the counter, may emit one inverted-bit1 notification,
/// runs the context boundary, and tail-clears exactly 24 bytes.
pub fn bt_stage69_bounded_callback<B: BtStage69Backend>(backend: &mut B) -> u32 {
    let source = backend.read_source_byte();
    let probe = backend.probe_boundary(u32::from(source)) as u8;

    let counter = backend.read_counter_byte();
    let limit = backend.read_limit_byte();
    let next = counter.wrapping_add(1);
    backend.write_counter_byte(next);

    if limit >= next {
        let is_zero = u32::from(probe == 0);
        return backend.notify_boundary(is_zero);
    }

    backend.write_counter_byte(0);
    let control = backend.read_control_byte();
    let bit1 = (control >> 1) & 1;
    if bit1 == probe {
        let inverted = u32::from(bit1 ^ 1);
        let _ = backend.notify_boundary(inverted);
    }

    let _ = backend.context_boundary(STAGE69_BT_CONTEXT_ADDR);
    backend.clear_boundary(
        STAGE69_BT_CONTEXT_ADDR,
        0,
        STAGE69_BT_CLEAR_BYTES,
    )
}

#[cfg(test)]
mod stage69_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        source: u8,
        probe: u32,
        counter: u8,
        limit: u8,
        control: u8,
        notify_return: u32,
        clear_return: u32,
        events: Vec<(&'static str, u32)>,
    }

    impl BtStage69Backend for B {
        fn read_source_byte(&mut self) -> u8 {
            self.events.push(("source", 0));
            self.source
        }
        fn probe_boundary(&mut self, source: u32) -> u32 {
            self.events.push(("probe", source));
            self.probe
        }
        fn read_counter_byte(&mut self) -> u8 {
            self.events.push(("counter", 0));
            self.counter
        }
        fn read_limit_byte(&mut self) -> u8 {
            self.events.push(("limit", 0));
            self.limit
        }
        fn write_counter_byte(&mut self, value: u8) {
            self.events.push(("write_counter", u32::from(value)));
            self.counter = value;
        }
        fn read_control_byte(&mut self) -> u8 {
            self.events.push(("control", 0));
            self.control
        }
        fn notify_boundary(&mut self, value: u32) -> u32 {
            self.events.push(("notify", value));
            self.notify_return
        }
        fn context_boundary(&mut self, context: u32) -> u32 {
            self.events.push(("context", context));
            0xAAAA_AAAA
        }
        fn clear_boundary(&mut self, context: u32, value: u32, len: u32) -> u32 {
            self.events.push(("clear_context", context));
            self.events.push(("clear_value", value));
            self.events.push(("clear_len", len));
            self.clear_return
        }
    }

    #[test]
    fn fast_path_writes_counter_before_zero_probe_notify_and_returns_notify_result() {
        let mut b = B { source: 9, probe: 0x100, counter: 1, limit: 2, notify_return: 0xDEAD_BEEF, ..Default::default() };
        assert_eq!(bt_stage69_bounded_callback(&mut b), 0xDEAD_BEEF);
        assert_eq!(b.counter, 2);
        assert_eq!(b.events, [
            ("source",0),("probe",9),("counter",0),("limit",0),
            ("write_counter",2),("notify",1),
        ]);
    }

    #[test]
    fn fast_path_nonzero_probe_notifies_zero() {
        let mut b = B { probe: 7, counter: 4, limit: 5, notify_return: 11, ..Default::default() };
        assert_eq!(bt_stage69_bounded_callback(&mut b), 11);
        assert_eq!(b.events.last(), Some(&("notify", 0)));
    }

    #[test]
    fn wrapped_counter_can_reenter_fast_path() {
        let mut b = B { probe: 1, counter: 255, limit: 0, notify_return: 3, ..Default::default() };
        assert_eq!(bt_stage69_bounded_callback(&mut b), 3);
        assert_eq!(b.counter, 0);
        assert!(!b.events.iter().any(|x| x.0 == "control"));
    }

    #[test]
    fn overflow_equal_bit1_emits_inverted_notify_then_context_and_clear() {
        let mut b = B { probe: 1, counter: 5, limit: 5, control: 0b10, notify_return: 99, clear_return: 0xCAFE_BABE, ..Default::default() };
        assert_eq!(bt_stage69_bounded_callback(&mut b), 0xCAFE_BABE);
        assert_eq!(b.counter, 0);
        assert_eq!(&b.events[b.events.len()-6..], [
            ("control",0),("notify",0),("context",STAGE69_BT_CONTEXT_ADDR),
            ("clear_context",STAGE69_BT_CONTEXT_ADDR),("clear_value",0),("clear_len",24),
        ]);
    }

    #[test]
    fn overflow_mismatch_skips_optional_notify() {
        let mut b = B { probe: 1, counter: 9, limit: 3, control: 0, clear_return: 5, ..Default::default() };
        assert_eq!(bt_stage69_bounded_callback(&mut b), 5);
        assert_eq!(b.events.iter().filter(|x| x.0 == "notify").count(), 0);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE69_CURRENT_BT_BOUNDED_CALLBACK_ADDR, 0x172660);
        assert_eq!(STAGE69_BT_SOURCE_BYTE_ADDR, 0x202FD4);
        assert_eq!(STAGE69_BT_COUNTER_ADDR, 0x223080);
        assert_eq!(STAGE69_BT_LIMIT_ADDR, 0x223084);
        assert_eq!(STAGE69_BT_CONTROL_BYTE_ADDR, 0x206800);
        assert_eq!(STAGE69_BT_CONTEXT_ADDR, 0x223088);
        assert_eq!(STAGE69_BT_PROBE_BOUNDARY, 0xBAC7C);
        assert_eq!(STAGE69_BT_NOTIFY_BOUNDARY, 0x72D00);
        assert_eq!(STAGE69_BT_CONTEXT_BOUNDARY, 0x151BC);
        assert_eq!(STAGE69_BT_CLEAR_BOUNDARY, 0x3D24);
        assert_eq!(STAGE69_BT_CLEAR_BYTES, 24);
    }
}

/// Stage 70: current masked-state publish wrapper at `0x16F4AA`.
///
/// The exact 88-byte current body has one relocation-normalized public-legacy
/// structural counterpart at `0x16C4DE`. Runtime meanings of opaque boundaries
/// remain deliberately unnamed.
pub const STAGE70_CURRENT_BT_MASKED_STATE_PUBLISH_ADDR: u32 = 0x0016_F4AA;
pub const STAGE70_BT_NORMALIZE_MODE_BOUNDARY: u32 = 0x0004_5624;
pub const STAGE70_BT_STAGE56_BOUNDARY: u32 = 0x0016_F438;
pub const STAGE70_BT_MAIN_BOUNDARY: u32 = 0x0004_4C00;
pub const STAGE70_BT_POST_PUBLISH_PREDICATE: u32 = 0x0004_6A08;
pub const STAGE70_BT_NOTIFY_BOUNDARY: u32 = 0x0004_6EB8;
pub const STAGE70_BT_PUBLISH_BASE: u32 = 0x0065_0160;
pub const STAGE70_BT_STATUS_BASE: u32 = 0x0020_6F78;
pub const STAGE70_BT_STATUS_BYTE_ADDR: u32 = 0x0020_6F8F;
pub const STAGE70_BT_MODE_THREE: u32 = 3;
pub const STAGE70_BT_STAGE56_TRIGGER_MODE: u32 = 25;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage70State {
    pub word0: u32,
    pub word4: u32,
}

pub trait BtStage70Backend {
    /// Current `0x45624(object, &mut mode)`; only used when mode is exactly 3.
    fn normalize_mode(&mut self, object: u32, mode: &mut u32) -> u32;
    /// Already recovered current Stage-56 boundary `0x16F438(object, 0)`.
    fn stage56_zero_mode(&mut self, object: u32, mode: u32) -> u32;
    /// Current `0x44C00(object, post-normalize-mode)`.
    fn main_boundary(&mut self, object: u32, mode: u32) -> u32;
    fn publish_word(&mut self, address: u32, value: u32);
    fn read_status_byte(&mut self) -> u8;
    /// Current `0x46A08`; exact local R0 forwarding is preserved.
    fn post_publish_predicate(&mut self, current_r0: u32) -> u32;
    /// Current `0x46EB8(4)`.
    fn notify_boundary(&mut self, value: u32) -> u32;
}

/// Safe source-level model of current `0x16F4AA`.
///
/// Local state masks happen before every boundary. Mode three may be rewritten
/// through a pointer by the first boundary and is then re-read. Post-normalize
/// mode 25 invokes Stage 56 with literal mode zero. The two masked words are
/// published after the main boundary. Return shape depends on the post-publish
/// status/predicate path exactly as in the current body.
pub fn bt_stage70_masked_state_publish<B: BtStage70Backend>(
    object: u32,
    mut mode: u32,
    state: &mut BtStage70State,
    backend: &mut B,
) -> u32 {
    state.word0 &= 0x003F_FFFF;
    state.word4 &= !0x0000_3F00;

    if mode == STAGE70_BT_MODE_THREE {
        let _ = backend.normalize_mode(object, &mut mode);
    }

    if mode == STAGE70_BT_STAGE56_TRIGGER_MODE {
        let _ = backend.stage56_zero_mode(object, 0);
    }

    let main_result = backend.main_boundary(object, mode);
    backend.publish_word(STAGE70_BT_PUBLISH_BASE, state.word0);
    backend.publish_word(STAGE70_BT_PUBLISH_BASE + 4, state.word4);

    if backend.read_status_byte() == 0 {
        return main_result;
    }

    let predicate = backend.post_publish_predicate(main_result);
    if predicate == 0 {
        return 0;
    }

    backend.notify_boundary(4)
}

#[cfg(test)]
mod stage70_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        normalized_mode: Option<u32>,
        main_return: u32,
        status: u8,
        predicate_return: u32,
        notify_return: u32,
        events: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage70Backend for B {
        fn normalize_mode(&mut self, object: u32, mode: &mut u32) -> u32 {
            self.events.push(("normalize", object, *mode));
            if let Some(v) = self.normalized_mode { *mode = v; }
            0xAAAA_AAAA
        }
        fn stage56_zero_mode(&mut self, object: u32, mode: u32) -> u32 {
            self.events.push(("stage56", object, mode));
            0xBBBB_BBBB
        }
        fn main_boundary(&mut self, object: u32, mode: u32) -> u32 {
            self.events.push(("main", object, mode));
            self.main_return
        }
        fn publish_word(&mut self, address: u32, value: u32) {
            self.events.push(("publish", address, value));
        }
        fn read_status_byte(&mut self) -> u8 {
            self.events.push(("status", STAGE70_BT_STATUS_BYTE_ADDR, 0));
            self.status
        }
        fn post_publish_predicate(&mut self, current_r0: u32) -> u32 {
            self.events.push(("predicate", current_r0, 0));
            self.predicate_return
        }
        fn notify_boundary(&mut self, value: u32) -> u32 {
            self.events.push(("notify", value, 0));
            self.notify_return
        }
    }

    #[test]
    fn mode_three_can_rewrite_to_twenty_five_before_stage56_and_main() {
        let mut s = BtStage70State { word0: 0xFFFF_FFFF, word4: 0xFFFF_FFFF };
        let mut b = B { normalized_mode: Some(25), main_return: 0x1234, status: 0, ..Default::default() };
        assert_eq!(bt_stage70_masked_state_publish(0x55, 3, &mut s, &mut b), 0x1234);
        assert_eq!(s.word0, 0x003F_FFFF);
        assert_eq!(s.word4, 0xFFFF_C0FF);
        assert_eq!(b.events, [
            ("normalize",0x55,3),("stage56",0x55,0),("main",0x55,25),
            ("publish",STAGE70_BT_PUBLISH_BASE,0x003F_FFFF),
            ("publish",STAGE70_BT_PUBLISH_BASE+4,0xFFFF_C0FF),
            ("status",STAGE70_BT_STATUS_BYTE_ADDR,0),
        ]);
    }

    #[test]
    fn mode_twenty_five_skips_normalizer_but_still_calls_stage56() {
        let mut s = BtStage70State::default();
        let mut b = B { main_return: 7, ..Default::default() };
        assert_eq!(bt_stage70_masked_state_publish(9, 25, &mut s, &mut b), 7);
        assert!(!b.events.iter().any(|x| x.0 == "normalize"));
        assert!(b.events.iter().any(|x| x.0 == "stage56"));
    }

    #[test]
    fn other_mode_skips_stage56_and_zero_status_preserves_main_return() {
        let mut s = BtStage70State { word0: 0xABCDEF12, word4: 0x12345678 };
        let mut b = B { main_return: 0xDEAD_BEEF, status: 0, ..Default::default() };
        assert_eq!(bt_stage70_masked_state_publish(1, 8, &mut s, &mut b), 0xDEAD_BEEF);
        assert!(!b.events.iter().any(|x| x.0 == "stage56"));
        assert!(!b.events.iter().any(|x| x.0 == "predicate"));
    }

    #[test]
    fn nonzero_status_zero_predicate_replaces_main_return_with_zero() {
        let mut s = BtStage70State::default();
        let mut b = B { main_return: 0xCAFE, status: 1, predicate_return: 0, notify_return: 99, ..Default::default() };
        assert_eq!(bt_stage70_masked_state_publish(2, 0, &mut s, &mut b), 0);
        assert!(b.events.contains(&("predicate", 0xCAFE, 0)));
        assert!(!b.events.iter().any(|x| x.0 == "notify"));
    }

    #[test]
    fn nonzero_predicate_notifies_literal_four_and_forwards_notify_return() {
        let mut s = BtStage70State::default();
        let mut b = B { main_return: 6, status: 1, predicate_return: 3, notify_return: 0xFACE_B00C, ..Default::default() };
        assert_eq!(bt_stage70_masked_state_publish(4, 1, &mut s, &mut b), 0xFACE_B00C);
        assert_eq!(b.events[b.events.len()-2..], [("predicate",6,0),("notify",4,0)]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE70_CURRENT_BT_MASKED_STATE_PUBLISH_ADDR, 0x16F4AA);
        assert_eq!(STAGE70_BT_NORMALIZE_MODE_BOUNDARY, 0x45624);
        assert_eq!(STAGE70_BT_STAGE56_BOUNDARY, 0x16F438);
        assert_eq!(STAGE70_BT_MAIN_BOUNDARY, 0x44C00);
        assert_eq!(STAGE70_BT_POST_PUBLISH_PREDICATE, 0x46A08);
        assert_eq!(STAGE70_BT_NOTIFY_BOUNDARY, 0x46EB8);
        assert_eq!(STAGE70_BT_PUBLISH_BASE, 0x650160);
        assert_eq!(STAGE70_BT_STATUS_BYTE_ADDR, 0x206F8F);
    }
}

/// Stage 71: current callback published by Stage 59 at raw Thumb pointer `0x16F511`.
///
/// The callable function starts at `0x16F510`. Its exact 118-byte current body has one
/// relocation-normalized public-legacy structural counterpart at `0x16C544`. The compiler
/// stack-canary word and failure sink are recorded as provenance/hardening boundaries; the
/// safe semantic model below focuses on the callback's observable dispatch behavior.
pub const STAGE71_CURRENT_BT_CALLBACK_ADDR: u32 = 0x0016_F510;
pub const STAGE71_CURRENT_BT_CALLBACK_THUMB: u32 = 0x0016_F511;
pub const STAGE71_BT_TABLE_BASE_ADDR: u32 = 0x0020_2A90;
pub const STAGE71_BT_GATE_ADDR: u32 = 0x0022_2708;
pub const STAGE71_BT_AMBIENT_WORD_ADDR: u32 = 0x0020_9644;
pub const STAGE71_BT_AMBIENT_BIT: u32 = 1 << 22;
pub const STAGE71_BT_GUARD_WORD_ADDR: u32 = 0x0020_0890;
pub const STAGE71_BT_STACK_GUARD_FAIL: u32 = ROM_STACK_GUARD_FAIL_ADDR;
pub const STAGE71_BT_PROBE_BOUNDARY: u32 = 0x0005_1800;
pub const STAGE71_BT_STAGE56_ADDR: u32 = STAGE56_CURRENT_BT_GATED_BIT22_UPDATE_ADDR;
pub const STAGE71_BT_MUTATE_HALFWORD_BOUNDARY: u32 = 0x0004_5624;
pub const STAGE71_BT_SELECTOR21_PREDICATE: u32 = 0x0008_917C;
pub const STAGE71_BT_REQUIRED_CLASS: u32 = 2;
pub const STAGE71_BT_SELECTOR_STAGE56: u32 = 6;
pub const STAGE71_BT_SELECTOR_GATE: u32 = 20;
pub const STAGE71_BT_SELECTOR_PREDICATE_GATE: u32 = 21;

pub trait BtStage71Backend {
    /// Exact unchecked firmware read `*(u16 *)(0x202A90 + selector * 2)`.
    ///
    /// It occurs before the class check, so the model deliberately does not add a bounds check.
    fn read_table_halfword(&mut self, selector: u32) -> u16;

    /// Current `0x51800` with the entry registers still carrying object/selector/class and the
    /// already-read table halfword in R3.
    fn probe(&mut self, object: u32, selector: u32, class: u32, table_halfword: u16) -> u32;

    /// Already recovered current Stage 56 called as `0x16F438(object, 0)`.
    fn stage56_zero_mode(&mut self, object: u32) -> u32;

    /// Current `0x45624(object, &mut local_halfword)`.
    fn mutate_halfword(&mut self, object: u32, value: &mut u16) -> u32;

    /// Current `0x8917C` on selector 21 with the entry register shape.
    fn selector21_predicate(
        &mut self,
        object: u32,
        selector: u32,
        class: u32,
        table_halfword: u16,
    ) -> u32;

    fn read_gate_byte(&mut self) -> u8;
    fn read_object_byte45(&mut self, object: u32) -> u8;
    fn read_ambient_word(&mut self) -> u32;
    fn write_ambient_word(&mut self, value: u32);
}

/// Safe source-level semantic model of current `0x16F510`.
///
/// The table halfword read is intentionally unconditional and unchecked. Only class two
/// dispatches selectors 6, 20, and 21. Selector 6 always returns one after its callback
/// sequence. Selectors 20/21 return zero and can only OR bit 22 into the ambient dword.
pub fn bt_stage71_published_callback<B: BtStage71Backend>(
    object: u32,
    selector: u32,
    class: u32,
    backend: &mut B,
) -> u32 {
    let mut local_halfword = backend.read_table_halfword(selector);

    if class != STAGE71_BT_REQUIRED_CLASS {
        return 0;
    }

    match selector {
        STAGE71_BT_SELECTOR_STAGE56 => {
            let probe = backend.probe(object, selector, class, local_halfword);
            if probe == 0 {
                let _ = backend.stage56_zero_mode(object);
            }
            let _ = backend.mutate_halfword(object, &mut local_halfword);
            1
        }
        STAGE71_BT_SELECTOR_GATE => {
            if backend.read_gate_byte() != 1 {
                return 0;
            }
            if backend.read_object_byte45(object) != 0 {
                return 0;
            }
            let old = backend.read_ambient_word();
            backend.write_ambient_word(old | STAGE71_BT_AMBIENT_BIT);
            0
        }
        STAGE71_BT_SELECTOR_PREDICATE_GATE => {
            if backend.selector21_predicate(object, selector, class, local_halfword) == 0 {
                return 0;
            }
            if backend.read_gate_byte() != 1 {
                return 0;
            }
            if backend.read_object_byte45(object) != 0 {
                return 0;
            }
            let old = backend.read_ambient_word();
            backend.write_ambient_word(old | STAGE71_BT_AMBIENT_BIT);
            0
        }
        _ => 0,
    }
}

#[cfg(test)]
mod stage71_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        table: u16,
        probe_result: u32,
        predicate_result: u32,
        gate: u8,
        byte45: u8,
        ambient: u32,
        mutated_value: u16,
        events: Vec<&'static str>,
        probe_args: (u32, u32, u32, u16),
        pred_args: (u32, u32, u32, u16),
    }

    impl BtStage71Backend for B {
        fn read_table_halfword(&mut self, _selector: u32) -> u16 {
            self.events.push("table");
            self.table
        }
        fn probe(&mut self, o: u32, s: u32, c: u32, h: u16) -> u32 {
            self.events.push("probe");
            self.probe_args = (o, s, c, h);
            self.probe_result
        }
        fn stage56_zero_mode(&mut self, _object: u32) -> u32 {
            self.events.push("stage56");
            0xAAAA_AAAA
        }
        fn mutate_halfword(&mut self, _object: u32, value: &mut u16) -> u32 {
            self.events.push("mutate");
            *value = 0xBEEF;
            self.mutated_value = *value;
            0xBBBB_BBBB
        }
        fn selector21_predicate(&mut self, o: u32, s: u32, c: u32, h: u16) -> u32 {
            self.events.push("predicate");
            self.pred_args = (o, s, c, h);
            self.predicate_result
        }
        fn read_gate_byte(&mut self) -> u8 {
            self.events.push("gate");
            self.gate
        }
        fn read_object_byte45(&mut self, _object: u32) -> u8 {
            self.events.push("byte45");
            self.byte45
        }
        fn read_ambient_word(&mut self) -> u32 {
            self.events.push("read_word");
            self.ambient
        }
        fn write_ambient_word(&mut self, value: u32) {
            self.events.push("write_word");
            self.ambient = value;
        }
    }

    #[test]
    fn table_read_precedes_class_rejection_and_no_bounds_rule_is_added() {
        let mut b = B { table: 0x1234, ..Default::default() };
        assert_eq!(bt_stage71_published_callback(1, 0xFFFF_FFFF, 7, &mut b), 0);
        assert_eq!(b.events, ["table"]);
    }

    #[test]
    fn selector_six_zero_probe_runs_stage56_then_mutates_and_returns_one() {
        let mut b = B { table: 0x2345, probe_result: 0, ..Default::default() };
        assert_eq!(bt_stage71_published_callback(0xABC, 6, 2, &mut b), 1);
        assert_eq!(b.events, ["table", "probe", "stage56", "mutate"]);
        assert_eq!(b.probe_args, (0xABC, 6, 2, 0x2345));
        assert_eq!(b.mutated_value, 0xBEEF);
    }

    #[test]
    fn selector_six_nonzero_probe_skips_stage56_but_still_mutates() {
        let mut b = B { table: 7, probe_result: 9, ..Default::default() };
        assert_eq!(bt_stage71_published_callback(4, 6, 2, &mut b), 1);
        assert_eq!(b.events, ["table", "probe", "mutate"]);
    }

    #[test]
    fn selector_twenty_gates_exactly_and_ors_only_bit22() {
        let mut b = B { table: 1, gate: 1, byte45: 0, ambient: 0xA501_0203, ..Default::default() };
        assert_eq!(bt_stage71_published_callback(5, 20, 2, &mut b), 0);
        assert_eq!(b.ambient, 0xA501_0203 | STAGE71_BT_AMBIENT_BIT);
        assert_eq!(b.events, ["table", "gate", "byte45", "read_word", "write_word"]);

        let mut blocked = B { gate: 0, ambient: 0x55, ..Default::default() };
        assert_eq!(bt_stage71_published_callback(5, 20, 2, &mut blocked), 0);
        assert_eq!(blocked.ambient, 0x55);
        assert_eq!(blocked.events, ["table", "gate"]);
    }

    #[test]
    fn selector_twenty_one_predicate_precedes_shared_gate_path() {
        let mut reject = B { predicate_result: 0, gate: 1, ..Default::default() };
        assert_eq!(bt_stage71_published_callback(9, 21, 2, &mut reject), 0);
        assert_eq!(reject.events, ["table", "predicate"]);

        let mut pass = B {
            table: 0x77,
            predicate_result: 1,
            gate: 1,
            byte45: 0,
            ambient: 0x10,
            ..Default::default()
        };
        assert_eq!(bt_stage71_published_callback(9, 21, 2, &mut pass), 0);
        assert_eq!(pass.pred_args, (9, 21, 2, 0x77));
        assert_eq!(pass.ambient, 0x10 | STAGE71_BT_AMBIENT_BIT);
        assert_eq!(pass.events, ["table", "predicate", "gate", "byte45", "read_word", "write_word"]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE71_CURRENT_BT_CALLBACK_ADDR, 0x16F510);
        assert_eq!(STAGE71_CURRENT_BT_CALLBACK_THUMB, 0x16F511);
        assert_eq!(STAGE71_BT_TABLE_BASE_ADDR, 0x202A90);
        assert_eq!(STAGE71_BT_GATE_ADDR, 0x222708);
        assert_eq!(STAGE71_BT_AMBIENT_WORD_ADDR, 0x209644);
        assert_eq!(STAGE71_BT_GUARD_WORD_ADDR, 0x200890);
        assert_eq!(STAGE71_BT_STACK_GUARD_FAIL, 0x94C0);
        assert_eq!(STAGE71_BT_PROBE_BOUNDARY, 0x51800);
        assert_eq!(STAGE71_BT_STAGE56_ADDR, 0x16F438);
        assert_eq!(STAGE71_BT_MUTATE_HALFWORD_BOUNDARY, 0x45624);
        assert_eq!(STAGE71_BT_SELECTOR21_PREDICATE, 0x8917C);
    }
}

/// Stage 72: raw callback-B entry published by Stage 59 at Thumb pointer `0x16F4A9`.
///
/// The callable entry starts at `0x16F4A8`. Its first instruction loads the stable ambient
/// state-pair pointer `0x209644` into R3 and then falls through into the already recovered
/// Stage-70 shared entry at `0x16F4AA`. The 90-byte callable region is therefore a concrete
/// wrapper-entry specialization of Stage 70 rather than a second independent routine.
pub const STAGE72_CURRENT_BT_CALLBACK_B_ADDR: u32 = 0x0016_F4A8;
pub const STAGE72_CURRENT_BT_CALLBACK_B_THUMB: u32 = 0x0016_F4A9;
pub const STAGE72_BT_AMBIENT_STATE_ADDR: u32 = 0x0020_9644;
pub const STAGE72_BT_SHARED_STAGE70_ADDR: u32 = STAGE70_CURRENT_BT_MASKED_STATE_PUBLISH_ADDR;

/// Safe source-level model of the current `0x16F4A8` callback entry.
///
/// `state` represents the exact ambient dword pair beginning at `0x209644`. The entry adds
/// no gate or mutation of its own: it supplies that fixed R3 pointer and immediately enters
/// Stage 70, so all return and call-order semantics are exactly Stage 70's.
pub fn bt_stage72_published_callback_b<B: BtStage70Backend>(
    object: u32,
    mode: u32,
    state: &mut BtStage70State,
    backend: &mut B,
) -> u32 {
    bt_stage70_masked_state_publish(object, mode, state, backend)
}

#[cfg(test)]
mod stage72_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        main_return: u32,
        events: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage70Backend for B {
        fn normalize_mode(&mut self, object: u32, mode: &mut u32) -> u32 {
            self.events.push(("normalize", object, *mode));
            0
        }
        fn stage56_zero_mode(&mut self, object: u32, mode: u32) -> u32 {
            self.events.push(("stage56", object, mode));
            0
        }
        fn main_boundary(&mut self, object: u32, mode: u32) -> u32 {
            self.events.push(("main", object, mode));
            self.main_return
        }
        fn publish_word(&mut self, address: u32, value: u32) {
            self.events.push(("publish", address, value));
        }
        fn read_status_byte(&mut self) -> u8 {
            self.events.push(("status", STAGE70_BT_STATUS_BYTE_ADDR, 0));
            0
        }
        fn post_publish_predicate(&mut self, current_r0: u32) -> u32 {
            self.events.push(("predicate", current_r0, 0));
            0
        }
        fn notify_boundary(&mut self, value: u32) -> u32 {
            self.events.push(("notify", value, 0));
            0
        }
    }

    #[test]
    fn callback_b_specializes_stage70_with_the_ambient_state_pair() {
        let mut state = BtStage70State { word0: 0xFFFF_FFFF, word4: 0xFFFF_FFFF };
        let mut b = B { main_return: 0x1234_5678, ..Default::default() };
        assert_eq!(bt_stage72_published_callback_b(0x55, 8, &mut state, &mut b), 0x1234_5678);
        assert_eq!(state.word0, 0x003F_FFFF);
        assert_eq!(state.word4, 0xFFFF_C0FF);
        assert_eq!(b.events, [
            ("main", 0x55, 8),
            ("publish", STAGE70_BT_PUBLISH_BASE, 0x003F_FFFF),
            ("publish", STAGE70_BT_PUBLISH_BASE + 4, 0xFFFF_C0FF),
            ("status", STAGE70_BT_STATUS_BYTE_ADDR, 0),
        ]);
    }

    #[test]
    fn provenance_constants_freeze_the_multi_entry_shape() {
        assert_eq!(STAGE72_CURRENT_BT_CALLBACK_B_ADDR, 0x16F4A8);
        assert_eq!(STAGE72_CURRENT_BT_CALLBACK_B_THUMB, 0x16F4A9);
        assert_eq!(STAGE72_BT_AMBIENT_STATE_ADDR, 0x209644);
        assert_eq!(STAGE72_BT_SHARED_STAGE70_ADDR, 0x16F4AA);
        assert_eq!(STAGE59_BT_CALLBACK_B_THUMB, STAGE72_CURRENT_BT_CALLBACK_B_THUMB);
    }
}

/// Stage 73: published callback-C periodic maintenance entry at `0x16F33C`.
///
/// Stage 59 publishes this routine as raw Thumb pointer `0x16F33D`. The exact current
/// body is 198 bytes and ends at the `POP {...,pc}` at `0x16F400`. Masking only the
/// nine four-byte direct-call encodings leaves 162 fixed bytes and yields exactly one
/// current hit plus one public-legacy structural counterpart at `0x16C370`.
///
/// The model preserves local memory ordering and arithmetic. The second modulo path uses
/// an unchecked hardware UDIV in firmware, so divisor-zero behavior is deliberately left
/// behind `unchecked_remainder` rather than inventing a source-level zero guard.
pub const STAGE73_CURRENT_BT_CALLBACK_C_ADDR: u32 = 0x0016_F33C;
pub const STAGE73_CURRENT_BT_CALLBACK_C_THUMB: u32 = 0x0016_F33D;
pub const STAGE73_BT_COUNTER_ADDR: u32 = 0x0020_9694;
pub const STAGE73_BT_PERIODIC_DIVISOR_ADDR: u32 = 0x0020_964C;
pub const STAGE73_BT_TRIGGER_ADDR: u32 = 0x0020_9596;
pub const STAGE73_BT_AUX_BASE_ADDR: u32 = 0x0020_2A06;
pub const STAGE73_BT_AUX_BYTE5_ADDR: u32 = 0x0020_2A0B;
pub const STAGE73_BT_COUNTDOWN_ADDR: u32 = 0x0020_963B;
pub const STAGE73_BT_MODULO_DIVISOR_ADDR: u32 = 0x0020_2A83;
pub const STAGE73_BT_NIBBLE_GATE_A_ADDR: u32 = 0x0020_9711;
pub const STAGE73_BT_NIBBLE_GATE_B_ADDR: u32 = 0x0020_9639;
pub const STAGE73_BT_STATUS_PTR_ADDR: u32 = 0x0020_3381;
pub const STAGE73_BT_SNAPSHOT_SOURCE_ADDR: u32 = 0x0065_0064;
pub const STAGE73_BT_SNAPSHOT_DEST_ADDR: u32 = 0x0020_959C;
pub const STAGE73_BT_PUBLISH_SOURCE_ADDR: u32 = 0x0022_2709;
pub const STAGE73_BT_PUBLISH_DEST_ADDR: u32 = 0x0020_A22A;

pub const STAGE73_BT_PERIODIC_FIRST_BOUNDARY: u32 = 0x0004_5DD4;
pub const STAGE73_BT_PERIODIC_SECOND_BOUNDARY: u32 = 0x0004_5F54;
pub const STAGE73_BT_TRIGGER_PRIMARY_BOUNDARY: u32 = 0x0004_51A0;
pub const STAGE73_BT_PREDICATE_SOURCE_BOUNDARY: u32 = 0x0004_69C0;
pub const STAGE73_BT_BOOLEAN_NOTIFY_BOUNDARY: u32 = 0x0004_6C48;
pub const STAGE73_BT_MODULO_BOUNDARY: u32 = 0x0004_58CC;
pub const STAGE73_BT_NIBBLE_BOUNDARY: u32 = 0x0004_5918;
pub const STAGE73_BT_STATUS_BOUNDARY: u32 = 0x0004_599C;
pub const STAGE73_BT_AUX_BOUNDARY: u32 = 0x0004_6C70;

pub trait BtStage73Backend {
    fn read_counter(&mut self) -> u32;
    fn write_counter(&mut self, value: u32);
    fn read_periodic_divisor(&mut self) -> u8;

    /// Current `0x45DD4(0, 3)` on the first periodic gate.
    fn periodic_first_boundary(&mut self, zero: u32, selector: u32) -> u32;
    /// Current `0x45F54`, with the first call's R0 forwarded directly.
    fn periodic_second_boundary(&mut self, forwarded_r0: u32) -> u32;

    fn read_trigger_byte(&mut self) -> u8;
    fn write_trigger_byte(&mut self, value: u8);
    fn trigger_primary_boundary(&mut self) -> u32;
    fn read_aux_byte5(&mut self) -> u8;
    fn predicate_source_boundary(&mut self) -> u32;
    fn boolean_notify_boundary(&mut self, value: u32) -> u32;

    fn read_countdown_byte(&mut self) -> u8;
    fn write_countdown_byte(&mut self, value: u8);

    fn read_modulo_divisor(&mut self) -> u8;
    /// Firmware performs UDIV/MLS without a local zero-divisor check on this path.
    /// The backend therefore owns the architecture/runtime contract for divisor zero.
    fn unchecked_remainder(&mut self, numerator: u32, divisor: u8) -> u32;
    fn modulo_boundary(&mut self) -> u32;

    fn read_nibble_gate_a(&mut self) -> u8;
    fn read_nibble_gate_b(&mut self) -> u8;
    fn nibble_boundary(&mut self) -> u32;

    fn read_status_object_ptr(&mut self) -> u32;
    fn read_status_word(&mut self, object: u32, offset: u32) -> u32;
    fn status_boundary(&mut self) -> u32;
    fn aux_boundary(&mut self) -> u32;

    fn read_snapshot_source_word(&mut self) -> u32;
    fn write_snapshot_word(&mut self, value: u32);
    fn read_publish_source_byte(&mut self) -> u8;
    fn write_publish_dest_byte(&mut self, value: u8);
}

/// Safe source-level model of current callback-C `0x16F33C`.
///
/// The routine always returns literal one. All opaque boundary returns except the first
/// periodic-call R0 forwarding and predicate-source equality test are ignored locally.
pub fn bt_stage73_published_callback_c<B: BtStage73Backend>(backend: &mut B) -> u32 {
    let counter = backend.read_counter().wrapping_add(1);
    backend.write_counter(counter);

    let periodic_divisor = backend.read_periodic_divisor();
    if periodic_divisor != 0
        && counter > 40
        && counter % u32::from(periodic_divisor) == 0
    {
        let forwarded = backend.periodic_first_boundary(0, 3);
        let _ = backend.periodic_second_boundary(forwarded);
    }

    if backend.read_trigger_byte() != 0 && counter == 3 {
        backend.write_trigger_byte(0);
        let _ = backend.trigger_primary_boundary();

        if backend.read_aux_byte5() != 0 {
            let predicate = backend.predicate_source_boundary();
            let exact_one = if predicate == 1 { 1 } else { 0 };
            let _ = backend.boolean_notify_boundary(exact_one);
        }
    }

    let countdown = backend.read_countdown_byte();
    if countdown != 0 {
        backend.write_countdown_byte(countdown.wrapping_sub(1));
    }

    let modulo_divisor = backend.read_modulo_divisor();
    if backend.unchecked_remainder(counter, modulo_divisor) == 0 {
        let _ = backend.modulo_boundary();
    }

    if counter & 0x0F == 0
        && backend.read_nibble_gate_a() != 0
        && backend.read_nibble_gate_b() != 0
    {
        let _ = backend.nibble_boundary();
    }

    let status_object = backend.read_status_object_ptr();
    let status_word = backend.read_status_word(status_object, 0x1C);
    if status_word & 0x10 != 0 {
        let _ = backend.status_boundary();
    }

    if backend.read_aux_byte5() != 0 {
        let _ = backend.aux_boundary();
    }

    if counter & 0x0F == 0 {
        let snapshot = backend.read_snapshot_source_word();
        backend.write_snapshot_word(snapshot);

        let nibble = (snapshot >> 12) & 0x0F;
        let value = if (1..=14).contains(&nibble) {
            1
        } else {
            backend.read_publish_source_byte()
        };
        backend.write_publish_dest_byte(value);
    }

    1
}

#[cfg(test)]
mod stage73_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        counter: u32,
        periodic_divisor: u8,
        trigger: u8,
        aux5: u8,
        countdown: u8,
        modulo_divisor: u8,
        remainder_result: u32,
        gate_a: u8,
        gate_b: u8,
        status_object: u32,
        status_word: u32,
        snapshot: u32,
        publish_source: u8,
        publish_dest: u8,
        periodic_first_return: u32,
        predicate_source_return: u32,
        events: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage73Backend for B {
        fn read_counter(&mut self) -> u32 { self.events.push(("read_counter",0,0)); self.counter }
        fn write_counter(&mut self, value: u32) { self.events.push(("write_counter",value,0)); self.counter=value; }
        fn read_periodic_divisor(&mut self) -> u8 { self.events.push(("periodic_divisor",0,0)); self.periodic_divisor }
        fn periodic_first_boundary(&mut self, zero:u32, selector:u32)->u32 {
            self.events.push(("periodic_first",zero,selector)); self.periodic_first_return
        }
        fn periodic_second_boundary(&mut self, forwarded_r0:u32)->u32 {
            self.events.push(("periodic_second",forwarded_r0,0)); 0
        }
        fn read_trigger_byte(&mut self)->u8 { self.events.push(("trigger_read",0,0)); self.trigger }
        fn write_trigger_byte(&mut self,value:u8){ self.events.push(("trigger_write",value as u32,0)); self.trigger=value; }
        fn trigger_primary_boundary(&mut self)->u32 { self.events.push(("trigger_primary",0,0)); 0 }
        fn read_aux_byte5(&mut self)->u8 { self.events.push(("aux5_read",0,0)); self.aux5 }
        fn predicate_source_boundary(&mut self)->u32 { self.events.push(("predicate_source",0,0)); self.predicate_source_return }
        fn boolean_notify_boundary(&mut self,value:u32)->u32 { self.events.push(("boolean_notify",value,0)); 0 }
        fn read_countdown_byte(&mut self)->u8 { self.events.push(("countdown_read",0,0)); self.countdown }
        fn write_countdown_byte(&mut self,value:u8){ self.events.push(("countdown_write",value as u32,0)); self.countdown=value; }
        fn read_modulo_divisor(&mut self)->u8 { self.events.push(("mod_divisor",0,0)); self.modulo_divisor }
        fn unchecked_remainder(&mut self,numerator:u32,divisor:u8)->u32 {
            self.events.push(("unchecked_remainder",numerator,divisor as u32)); self.remainder_result
        }
        fn modulo_boundary(&mut self)->u32 { self.events.push(("modulo_boundary",0,0)); 0 }
        fn read_nibble_gate_a(&mut self)->u8 { self.events.push(("gate_a",0,0)); self.gate_a }
        fn read_nibble_gate_b(&mut self)->u8 { self.events.push(("gate_b",0,0)); self.gate_b }
        fn nibble_boundary(&mut self)->u32 { self.events.push(("nibble_boundary",0,0)); 0 }
        fn read_status_object_ptr(&mut self)->u32 { self.events.push(("status_ptr",0,0)); self.status_object }
        fn read_status_word(&mut self,object:u32,offset:u32)->u32 {
            self.events.push(("status_word",object,offset)); self.status_word
        }
        fn status_boundary(&mut self)->u32 { self.events.push(("status_boundary",0,0)); 0 }
        fn aux_boundary(&mut self)->u32 { self.events.push(("aux_boundary",0,0)); 0 }
        fn read_snapshot_source_word(&mut self)->u32 { self.events.push(("snapshot_read",0,0)); self.snapshot }
        fn write_snapshot_word(&mut self,value:u32){ self.events.push(("snapshot_write",value,0)); }
        fn read_publish_source_byte(&mut self)->u8 { self.events.push(("publish_source",0,0)); self.publish_source }
        fn write_publish_dest_byte(&mut self,value:u8){ self.events.push(("publish_dest",value as u32,0)); self.publish_dest=value; }
    }

    #[test]
    fn periodic_gate_forwards_first_return_into_second_boundary() {
        let mut b=B{counter:40,periodic_divisor:41,modulo_divisor:7,remainder_result:1,
                    periodic_first_return:0x1234_5678,..Default::default()};
        assert_eq!(bt_stage73_published_callback_c(&mut b),1);
        assert_eq!(b.counter,41);
        assert!(b.events.contains(&("periodic_first",0,3)));
        assert!(b.events.contains(&("periodic_second",0x1234_5678,0)));
    }

    #[test]
    fn trigger_three_clears_before_calls_and_exact_one_notifies_one() {
        let mut b=B{counter:2,periodic_divisor:0,trigger:1,aux5:1,countdown:2,
                    modulo_divisor:5,remainder_result:1,predicate_source_return:1,..Default::default()};
        assert_eq!(bt_stage73_published_callback_c(&mut b),1);
        assert_eq!(b.counter,3);
        assert_eq!(b.trigger,0);
        assert_eq!(b.countdown,1);
        let tw=b.events.iter().position(|x|x.0=="trigger_write").unwrap();
        let tp=b.events.iter().position(|x|x.0=="trigger_primary").unwrap();
        let ps=b.events.iter().position(|x|x.0=="predicate_source").unwrap();
        let bn=b.events.iter().position(|x|x.0=="boolean_notify").unwrap();
        assert!(tw<tp && tp<ps && ps<bn);
        assert_eq!(b.events[bn],("boolean_notify",1,0));
    }

    #[test]
    fn predicate_values_other_than_one_notify_zero() {
        let mut b=B{counter:2,trigger:1,aux5:1,modulo_divisor:2,remainder_result:1,
                    predicate_source_return:2,..Default::default()};
        let _=bt_stage73_published_callback_c(&mut b);
        assert!(b.events.contains(&("boolean_notify",0,0)));
    }

    #[test]
    fn zero_modulo_divisor_is_forwarded_without_inventing_a_guard() {
        let mut b=B{counter:6,modulo_divisor:0,remainder_result:0,..Default::default()};
        assert_eq!(bt_stage73_published_callback_c(&mut b),1);
        assert!(b.events.contains(&("unchecked_remainder",7,0)));
        assert!(b.events.contains(&("modulo_boundary",0,0)));
    }

    #[test]
    fn low_nibble_status_aux_and_snapshot_paths_preserve_order_and_value_rule() {
        let mut b=B{counter:15,modulo_divisor:3,remainder_result:1,gate_a:1,gate_b:1,
                    aux5:1,status_object:0x5000,status_word:0x10,snapshot:0x0000_A000,
                    publish_source:0x5A,..Default::default()};
        assert_eq!(bt_stage73_published_callback_c(&mut b),1);
        assert!(b.events.contains(&("nibble_boundary",0,0)));
        assert!(b.events.contains(&("status_word",0x5000,0x1C)));
        assert!(b.events.contains(&("status_boundary",0,0)));
        assert!(b.events.contains(&("aux_boundary",0,0)));
        assert_eq!(b.publish_dest,1);
        assert!(!b.events.iter().any(|x|x.0=="publish_source"));
    }

    #[test]
    fn edge_nibbles_zero_and_fifteen_copy_publish_source_byte() {
        for snapshot in [0u32,0x0000_F000] {
            let mut b=B{counter:15,modulo_divisor:7,remainder_result:1,snapshot,
                        publish_source:0xA6,..Default::default()};
            assert_eq!(bt_stage73_published_callback_c(&mut b),1);
            assert_eq!(b.publish_dest,0xA6);
            assert!(b.events.iter().any(|x|x.0=="publish_source"));
        }
    }

    #[test]
    fn provenance_constants_close_stage59_callback_c() {
        assert_eq!(STAGE73_CURRENT_BT_CALLBACK_C_ADDR,0x16F33C);
        assert_eq!(STAGE73_CURRENT_BT_CALLBACK_C_THUMB,0x16F33D);
        assert_eq!(STAGE59_BT_CALLBACK_C_THUMB,STAGE73_CURRENT_BT_CALLBACK_C_THUMB);
        assert_eq!(STAGE73_BT_COUNTER_ADDR,0x209694);
        assert_eq!(STAGE73_BT_PUBLISH_SOURCE_ADDR,0x222709);
        assert_eq!(STAGE73_BT_PUBLISH_DEST_ADDR,0x20A22A);
    }
}

/// Stage 74: current signed-threshold selector at `0x16F820`.
///
/// The exact current body is 76 bytes, contains no runtime calls and no literal loads,
/// and has exactly one hit in the current executable range. No exact public-legacy body
/// counterpart exists, so this reconstruction is current-HCD-first.
///
/// Firmware reads a record pointer from `context + 0x94 + selector*4`, scans record bytes
/// 1 through 7 as signed i8 thresholds, and on the first threshold below the signed 32-bit
/// target chooses the current or previous index using the exact signed-8-bit distance
/// arithmetic. If no threshold qualifies it returns seven.
pub const STAGE74_CURRENT_BT_SIGNED_THRESHOLD_SELECTOR_ADDR: u32 = 0x0016_F820;
pub const STAGE74_BT_TABLE_PTR_OFFSET: u32 = 0x94;
pub const STAGE74_BT_FIRST_INDEX: u8 = 1;
pub const STAGE74_BT_SENTINEL_INDEX: u8 = 7;
pub const STAGE74_BT_SCAN_LIMIT: u8 = 8;

pub trait BtStage74Backend {
    fn read_record_ptr(&mut self, address: u32) -> u32;
    fn read_byte(&mut self, address: u32) -> u8;
}

/// Safe source-level model of current `0x16F820`.
///
/// The initial threshold comparison uses the full incoming R0 as signed i32. Only after a
/// qualifying threshold is found does firmware truncate R0 to u8 for the distance math.
/// That awkward split is preserved intentionally.
pub fn bt_stage74_signed_threshold_selector<B: BtStage74Backend>(
    target: u32,
    selector: u32,
    context: u32,
    backend: &mut B,
) -> u32 {
    let cell = context
        .wrapping_add(selector.wrapping_shl(2))
        .wrapping_add(STAGE74_BT_TABLE_PTR_OFFSET);
    let record = backend.read_record_ptr(cell);
    let mut scan = record.wrapping_add(1);
    let mut index = STAGE74_BT_FIRST_INDEX;

    loop {
        let scan_byte = backend.read_byte(scan);
        scan = scan.wrapping_add(1);
        let scan_signed = i32::from(scan_byte as i8);

        if scan_signed >= target as i32 {
            index = index.wrapping_add(1);
            if index == STAGE74_BT_SCAN_LIMIT {
                return u32::from(STAGE74_BT_SENTINEL_INDEX);
            }
            continue;
        }

        let base_byte = backend.read_byte(record);
        let target_low = target as u8;

        let mut base_distance = target_low.wrapping_sub(base_byte) as i8;
        if base_distance < 0 {
            base_distance = base_distance.wrapping_neg();
        }

        let scan_distance = target_low.wrapping_sub(scan_byte) as i8;
        if base_distance > scan_distance {
            return u32::from(index);
        }
        return u32::from(index.wrapping_sub(1));
    }
}

#[cfg(test)]
mod stage74_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    struct B {
        cell: u32,
        record: u32,
        bytes: [u8; 8],
        reads: Vec<(u32, u8)>,
    }

    impl B {
        fn new(record: u32, bytes: [u8; 8]) -> Self {
            Self { cell: 0, record, bytes, reads: Vec::new() }
        }
    }

    impl BtStage74Backend for B {
        fn read_record_ptr(&mut self, address: u32) -> u32 {
            self.cell = address;
            self.record
        }
        fn read_byte(&mut self, address: u32) -> u8 {
            let offset = address.wrapping_sub(self.record) as usize;
            let value = self.bytes[offset];
            self.reads.push((address, value));
            value
        }
    }

    #[test]
    fn pointer_cell_uses_context_plus_selector_times_four_plus_0x94() {
        let mut b = B::new(0x5000, [0, 20, 20, 20, 20, 20, 20, 20]);
        assert_eq!(bt_stage74_signed_threshold_selector(10, 3, 0x1000, &mut b), 7);
        assert_eq!(b.cell, 0x1000 + 3 * 4 + 0x94);
    }

    #[test]
    fn no_qualifying_threshold_reads_only_bytes_one_through_seven_and_returns_seven() {
        let mut b = B::new(0x6000, [0xAA, 10, 11, 12, 13, 14, 15, 16]);
        assert_eq!(bt_stage74_signed_threshold_selector(5, 0, 0, &mut b), 7);
        assert_eq!(b.reads.len(), 7);
        assert_eq!(b.reads[0].0, 0x6001);
        assert_eq!(b.reads[6].0, 0x6007);
        assert!(!b.reads.iter().any(|x|x.0 == 0x6000));
    }

    #[test]
    fn qualifying_first_threshold_can_choose_previous_index() {
        let mut b = B::new(0x7000, [8, 5, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bt_stage74_signed_threshold_selector(10, 0, 0, &mut b), 0);
        assert_eq!(b.reads.as_slice(), &[(0x7001, 5), (0x7000, 8)]);
    }

    #[test]
    fn qualifying_threshold_can_choose_current_index() {
        let mut b = B::new(0x7100, [0, 9, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bt_stage74_signed_threshold_selector(10, 0, 0, &mut b), 1);
    }

    #[test]
    fn full_signed_target_is_used_before_low_byte_distance_math() {
        let mut b = B::new(0x7200, [0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(bt_stage74_signed_threshold_selector(u32::MAX, 0, 0, &mut b), 7);
        assert_eq!(b.reads.len(), 7);
    }

    #[test]
    fn negative_base_distance_uses_wrapping_i8_absolute_shape() {
        let mut b = B::new(0x7300, [5, 0xF9, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bt_stage74_signed_threshold_selector(250, 0, 0, &mut b), 1);
    }

    #[test]
    fn provenance_constants_are_current_only() {
        assert_eq!(STAGE74_CURRENT_BT_SIGNED_THRESHOLD_SELECTOR_ADDR, 0x16F820);
        assert_eq!(STAGE74_BT_TABLE_PTR_OFFSET, 0x94);
        assert_eq!(STAGE74_BT_FIRST_INDEX, 1);
        assert_eq!(STAGE74_BT_SENTINEL_INDEX, 7);
        assert_eq!(STAGE74_BT_SCAN_LIMIT, 8);
    }
}

/// Stage 75: current conditional publish selector at `0x16F874`.
///
/// The exact current 76-byte body is unique in the current Orange Pi HCD. The
/// public legacy HCD does not contain a relocation-normalized structural
/// counterpart, so Stage 75 is current-HCD-first only.
///
/// A candidate byte at `incoming_r1 + incoming_r2 + 4` is read before the
/// context type is inspected. Type `0x13` replaces that candidate through an
/// opaque boundary plus a sign-bit-selected addressing path. The selected
/// value is published as a dword to the primary output, and optionally to a
/// secondary output when the gate byte is nonzero.
pub const STAGE75_CURRENT_BT_CONDITIONAL_PUBLISH_ADDR: u32 = 0x0016_F874;
pub const STAGE75_BT_CONTEXT_ROOT_BASE: u32 = 0x0020_6EA0;
pub const STAGE75_BT_CONTEXT_PTR_ADDR: u32 = STAGE75_BT_CONTEXT_ROOT_BASE + 8;
pub const STAGE75_BT_SPECIAL_BOUNDARY: u32 = 0x0008_8414;
pub const STAGE75_BT_PRIMARY_OUTPUT_ADDR: u32 = 0x0060_019C;
pub const STAGE75_BT_SECONDARY_GATE_ADDR: u32 = 0x0020_B265;
pub const STAGE75_BT_SECONDARY_OUTPUT_ADDR: u32 = 0x0060_0164;
pub const STAGE75_BT_SPECIAL_TYPE: u8 = 0x13;
pub const STAGE75_BT_SPECIAL_EARLY_MASK: u32 = 0x80;

pub trait BtStage75Backend {
    /// Byte read from an arbitrary current-memory address.
    fn read_memory_byte(&mut self, address: u32) -> u8;
    /// Dword at `0x206EA8`.
    fn read_context_ptr(&mut self) -> u32;
    fn read_context_byte(&mut self, context: u32, offset: u32) -> u8;
    fn read_context_halfword(&mut self, context: u32, offset: u32) -> u16;

    /// Current opaque `0x88414(context + 0x28)`.
    fn special_boundary(&mut self, argument: u32) -> u32;

    fn write_primary_output(&mut self, value: u32);
    fn read_secondary_gate(&mut self) -> u8;
    fn write_secondary_output(&mut self, value: u32);
}

/// Safe source-level model of current `0x16F874`.
///
/// Ordering is intentionally explicit:
/// 1. Read the candidate byte from `r1 + r2 + 4`.
/// 2. Load the context pointer and type byte.
/// 3. For non-`0x13`, publish the original candidate and return incoming R0.
/// 4. For `0x13`, call the opaque boundary on `context + 0x28` and mask its
///    return with `0xF0`. Masked `0x80` returns immediately without outputs.
/// 5. Otherwise choose the published byte by context byte `+0x27A` bit 7:
///    clear => `[incoming_r0 + incoming_r2 + 0x14]`;
///    set => `[incoming_r0 + halfword(context+0x258) + 0x24]`.
/// 6. Publish primary first, then read the secondary gate, then optionally
///    publish secondary. The function returns the masked boundary result.
pub fn bt_stage75_conditional_publish<B: BtStage75Backend>(
    incoming_r0: u32,
    incoming_r1: u32,
    incoming_r2: u32,
    backend: &mut B,
) -> u32 {
    let candidate_addr = incoming_r1
        .wrapping_add(incoming_r2)
        .wrapping_add(4);
    let mut selected = u32::from(backend.read_memory_byte(candidate_addr));

    let context = backend.read_context_ptr();
    let ty = backend.read_context_byte(context, 0x10);

    let result = if ty == STAGE75_BT_SPECIAL_TYPE {
        let masked = backend
            .special_boundary(context.wrapping_add(0x28))
            & 0xF0;
        if masked == STAGE75_BT_SPECIAL_EARLY_MASK {
            return masked;
        }

        let sign_byte = backend.read_context_byte(context, 0x27A);
        let address = if (sign_byte & 0x80) == 0 {
            incoming_r0
                .wrapping_add(incoming_r2)
                .wrapping_add(0x14)
        } else {
            incoming_r0
                .wrapping_add(u32::from(
                    backend.read_context_halfword(context, 0x258),
                ))
                .wrapping_add(0x24)
        };
        selected = u32::from(backend.read_memory_byte(address));
        masked
    } else {
        incoming_r0
    };

    backend.write_primary_output(selected);
    if backend.read_secondary_gate() != 0 {
        backend.write_secondary_output(selected);
    }
    result
}

#[cfg(test)]
mod stage75_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        context: u32,
        ty: u8,
        sign_byte: u8,
        halfword: u16,
        ordinary_addr: u32,
        ordinary_value: u8,
        special_addr: u32,
        special_value: u8,
        boundary_return: u32,
        gate: u8,
        events: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage75Backend for B {
        fn read_memory_byte(&mut self, address: u32) -> u8 {
            self.events.push(("read_mem", address, 0));
            if address == self.ordinary_addr {
                self.ordinary_value
            } else if address == self.special_addr {
                self.special_value
            } else {
                panic!("unexpected memory byte address {address:#x}")
            }
        }

        fn read_context_ptr(&mut self) -> u32 {
            self.events.push(("context_ptr", STAGE75_BT_CONTEXT_PTR_ADDR, 0));
            self.context
        }

        fn read_context_byte(&mut self, context: u32, offset: u32) -> u8 {
            self.events.push(("context_byte", context, offset));
            match offset {
                0x10 => self.ty,
                0x27A => self.sign_byte,
                _ => panic!("unexpected context byte offset {offset:#x}"),
            }
        }

        fn read_context_halfword(&mut self, context: u32, offset: u32) -> u16 {
            self.events.push(("context_half", context, offset));
            assert_eq!(offset, 0x258);
            self.halfword
        }

        fn special_boundary(&mut self, argument: u32) -> u32 {
            self.events.push(("boundary", argument, 0));
            self.boundary_return
        }

        fn write_primary_output(&mut self, value: u32) {
            self.events.push(("primary", STAGE75_BT_PRIMARY_OUTPUT_ADDR, value));
        }

        fn read_secondary_gate(&mut self) -> u8 {
            self.events.push(("gate", STAGE75_BT_SECONDARY_GATE_ADDR, 0));
            self.gate
        }

        fn write_secondary_output(&mut self, value: u32) {
            self.events.push(("secondary", STAGE75_BT_SECONDARY_OUTPUT_ADDR, value));
        }
    }

    #[test]
    fn ordinary_path_reads_candidate_before_type_and_returns_incoming_r0() {
        let mut b = B {
            context: 0x1000,
            ty: 0x12,
            ordinary_addr: 0x2000 + 0x30 + 4,
            ordinary_value: 0xA5,
            gate: 0,
            ..Default::default()
        };
        assert_eq!(bt_stage75_conditional_publish(0xCAFE_BABE, 0x2000, 0x30, &mut b), 0xCAFE_BABE);
        assert_eq!(b.events, [
            ("read_mem", 0x2034, 0),
            ("context_ptr", STAGE75_BT_CONTEXT_PTR_ADDR, 0),
            ("context_byte", 0x1000, 0x10),
            ("primary", STAGE75_BT_PRIMARY_OUTPUT_ADDR, 0xA5),
            ("gate", STAGE75_BT_SECONDARY_GATE_ADDR, 0),
        ]);
    }

    #[test]
    fn special_mask_80_returns_early_after_candidate_read_without_outputs() {
        let mut b = B {
            context: 0x5000,
            ty: STAGE75_BT_SPECIAL_TYPE,
            ordinary_addr: 0x1024,
            ordinary_value: 9,
            boundary_return: 0x18F,
            ..Default::default()
        };
        assert_eq!(bt_stage75_conditional_publish(7, 0x1000, 0x20, &mut b), 0x80);
        assert_eq!(b.events, [
            ("read_mem", 0x1024, 0),
            ("context_ptr", STAGE75_BT_CONTEXT_PTR_ADDR, 0),
            ("context_byte", 0x5000, 0x10),
            ("boundary", 0x5028, 0),
        ]);
    }

    #[test]
    fn special_nonnegative_flag_uses_incoming_r2_base_and_optional_secondary_publish() {
        let mut b = B {
            context: 0x7000,
            ty: STAGE75_BT_SPECIAL_TYPE,
            sign_byte: 0x7F,
            ordinary_addr: 0x3044,
            ordinary_value: 1,
            special_addr: 0x4000 + 0x40 + 0x14,
            special_value: 0x5A,
            boundary_return: 0x12F,
            gate: 1,
            ..Default::default()
        };
        assert_eq!(bt_stage75_conditional_publish(0x4000, 0x3000, 0x40, &mut b), 0x20);
        assert_eq!(b.events, [
            ("read_mem", 0x3044, 0),
            ("context_ptr", STAGE75_BT_CONTEXT_PTR_ADDR, 0),
            ("context_byte", 0x7000, 0x10),
            ("boundary", 0x7028, 0),
            ("context_byte", 0x7000, 0x27A),
            ("read_mem", 0x4054, 0),
            ("primary", STAGE75_BT_PRIMARY_OUTPUT_ADDR, 0x5A),
            ("gate", STAGE75_BT_SECONDARY_GATE_ADDR, 0),
            ("secondary", STAGE75_BT_SECONDARY_OUTPUT_ADDR, 0x5A),
        ]);
    }

    #[test]
    fn special_negative_flag_uses_context_halfword_offset() {
        let mut b = B {
            context: 0x9000,
            ty: STAGE75_BT_SPECIAL_TYPE,
            sign_byte: 0x80,
            halfword: 0x123,
            ordinary_addr: 0x1118,
            ordinary_value: 2,
            special_addr: 0x8000 + 0x123 + 0x24,
            special_value: 0x6B,
            boundary_return: 0x31,
            gate: 0,
            ..Default::default()
        };
        assert_eq!(bt_stage75_conditional_publish(0x8000, 0x1100, 0x14, &mut b), 0x30);
        assert!(b.events.contains(&("context_half", 0x9000, 0x258)));
        assert!(b.events.contains(&("read_mem", 0x8147, 0)));
        assert!(b.events.contains(&("primary", STAGE75_BT_PRIMARY_OUTPUT_ADDR, 0x6B)));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE75_CURRENT_BT_CONDITIONAL_PUBLISH_ADDR, 0x16F874);
        assert_eq!(STAGE75_BT_CONTEXT_PTR_ADDR, 0x206EA8);
        assert_eq!(STAGE75_BT_SPECIAL_BOUNDARY, 0x88414);
        assert_eq!(STAGE75_BT_PRIMARY_OUTPUT_ADDR, 0x60019C);
        assert_eq!(STAGE75_BT_SECONDARY_GATE_ADDR, 0x20B265);
        assert_eq!(STAGE75_BT_SECONDARY_OUTPUT_ADDR, 0x600164);
    }
}

/// Stage 76: current indirect callback orchestrator at `0x16F970`.
///
/// The exact current 160-byte body is unique in the current Orange Pi HCD and
/// contains no direct BL/B.W runtime call. All runtime calls are indirect BLX
/// through current-memory tables. Public legacy does not provide an exact
/// structural counterpart, so semantics are current-HCD-first.
pub const STAGE76_CURRENT_BT_INDIRECT_ORCHESTRATOR_ADDR: u32 = 0x0016_F970;
pub const STAGE76_BT_ENABLE_FLAGS_ADDR: u32 = 0x0020_1AF4;
pub const STAGE76_BT_CALLBACK_TABLE_ROOT_ADDR: u32 = 0x0020_3488;
pub const STAGE76_BT_MODE_ADDR: u32 = 0x0020_DAA5;
pub const STAGE76_BT_REQUIRED_TABLE_BASE: u32 = 0x0020_DA8C;
pub const STAGE76_BT_PROVIDER_ROOT_ADDR: u32 = 0x0020_375C;
pub const STAGE76_BT_REQUEST_BYTE_ADDR: u32 = 0x0020_D9A2;
pub const STAGE76_BT_STATUS_BYTE_ADDR: u32 = 0x0020_D9A4;

pub trait BtStage76Backend {
    fn read_enable_flags(&mut self) -> u8;
    fn read_callback_table_root(&mut self) -> u32;
    fn read_mode_byte(&mut self) -> u8;
    fn read_required_word(&mut self, index: u32) -> u32;
    fn read_provider_root(&mut self) -> u32;
    fn write_request_byte(&mut self, value: u8);
    fn write_status_byte(&mut self, value: u8);

    /// Read a function-pointer dword at `base + offset`.
    fn read_indirect_ptr(&mut self, base: u32, offset: u32) -> u32;

    /// Invoke the raw indirect target with the currently-live R0 token.
    fn call_indirect(&mut self, target: u32, current_r0: u32) -> u32;
}

fn bt_stage76_optional_call<B: BtStage76Backend>(
    backend: &mut B,
    offset: u32,
    current_r0: u32,
) -> u32 {
    let table = backend.read_callback_table_root();
    let target = backend.read_indirect_ptr(table, offset);
    if target == 0 {
        current_r0
    } else {
        backend.call_indirect(target, current_r0)
    }
}

/// Safe source-level model of current `0x16F970`.
///
/// Early local gates return the incoming R0 unchanged. The provider call
/// through `[(*0x20375C) + 8]` has no local null check; its zero return exits
/// with zero. Every optional callback re-reads the callback-table root from
/// `0x203488`, preserving table replacement/mutation visibility. The current
/// R0 token flows through each callback that is actually invoked.
///
/// The mode byte is read twice: once for the three-word preflight requirement
/// and again after slots 0/4/8 have run, so callback mutations can change the
/// branch. The final slot `+0x28` is a tail-call when nonzero.
pub fn bt_stage76_indirect_orchestrator<B: BtStage76Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    if (backend.read_enable_flags() & 1) == 0 {
        return incoming_r0;
    }

    if backend.read_callback_table_root() == 0 {
        return incoming_r0;
    }

    if backend.read_mode_byte() != 0 {
        let mut index = 0u32;
        while index < 3 {
            if backend.read_required_word(index) == 0 {
                return incoming_r0;
            }
            index += 1;
        }
    }

    let provider = backend.read_provider_root();
    backend.write_request_byte(incoming_r0 as u8);

    // Firmware performs this BLX without a local null test.
    let provider_target = backend.read_indirect_ptr(provider, 8);
    let mut current = backend.call_indirect(provider_target, incoming_r0);
    if current == 0 {
        return 0;
    }

    current = bt_stage76_optional_call(backend, 0x00, current);
    backend.write_status_byte(2);
    current = bt_stage76_optional_call(backend, 0x04, current);
    current = bt_stage76_optional_call(backend, 0x08, current);

    if backend.read_mode_byte() != 0 {
        current = bt_stage76_optional_call(backend, 0x0C, current);
        current = bt_stage76_optional_call(backend, 0x10, current);
    } else {
        current = bt_stage76_optional_call(backend, 0x14, current);
        current = bt_stage76_optional_call(backend, 0x18, current);
        current = bt_stage76_optional_call(backend, 0x1C, current);
        current = bt_stage76_optional_call(backend, 0x20, current);
        current = bt_stage76_optional_call(backend, 0x24, current);
    }

    // Final slot is loaded after all prior callbacks. A null pointer returns
    // the currently-live R0; a nonnull pointer is tail-called.
    let table = backend.read_callback_table_root();
    let tail = backend.read_indirect_ptr(table, 0x28);
    if tail == 0 {
        current
    } else {
        backend.call_indirect(tail, current)
    }
}

#[cfg(test)]
mod stage76_tests {
    extern crate std;
    use super::*;
    use std::collections::BTreeMap;
    use std::vec::Vec;

    struct B {
        enable: u8,
        mode_reads: Vec<u8>,
        mode_index: usize,
        roots: Vec<u32>,
        root_index: usize,
        required: [u32; 3],
        provider: u32,
        ptrs: BTreeMap<(u32, u32), u32>,
        returns: BTreeMap<u32, u32>,
        events: Vec<(&'static str, u32, u32)>,
    }

    impl Default for B {
        fn default() -> Self {
            Self {
                enable: 0,
                mode_reads: std::vec![0],
                mode_index: 0,
                roots: std::vec![0x1000],
                root_index: 0,
                required: [1; 3],
                provider: 0x2000,
                ptrs: BTreeMap::new(),
                returns: BTreeMap::new(),
                events: Vec::new(),
            }
        }
    }

    impl B {
        fn mode(&mut self) -> u8 {
            let i = core::cmp::min(self.mode_index, self.mode_reads.len() - 1);
            self.mode_index += 1;
            self.mode_reads[i]
        }

        fn root(&mut self) -> u32 {
            let i = core::cmp::min(self.root_index, self.roots.len() - 1);
            self.root_index += 1;
            self.roots[i]
        }
    }

    impl BtStage76Backend for B {
        fn read_enable_flags(&mut self) -> u8 {
            self.events.push(("enable", STAGE76_BT_ENABLE_FLAGS_ADDR, 0));
            self.enable
        }
        fn read_callback_table_root(&mut self) -> u32 {
            let value = self.root();
            self.events.push(("root", STAGE76_BT_CALLBACK_TABLE_ROOT_ADDR, value));
            value
        }
        fn read_mode_byte(&mut self) -> u8 {
            let value = self.mode();
            self.events.push(("mode", STAGE76_BT_MODE_ADDR, value as u32));
            value
        }
        fn read_required_word(&mut self, index: u32) -> u32 {
            let value = self.required[index as usize];
            self.events.push(("required", index, value));
            value
        }
        fn read_provider_root(&mut self) -> u32 {
            self.events.push(("provider", STAGE76_BT_PROVIDER_ROOT_ADDR, self.provider));
            self.provider
        }
        fn write_request_byte(&mut self, value: u8) {
            self.events.push(("request", STAGE76_BT_REQUEST_BYTE_ADDR, value as u32));
        }
        fn write_status_byte(&mut self, value: u8) {
            self.events.push(("status", STAGE76_BT_STATUS_BYTE_ADDR, value as u32));
        }
        fn read_indirect_ptr(&mut self, base: u32, offset: u32) -> u32 {
            let value = *self.ptrs.get(&(base, offset)).unwrap_or(&0);
            self.events.push(("ptr", base.wrapping_add(offset), value));
            value
        }
        fn call_indirect(&mut self, target: u32, current_r0: u32) -> u32 {
            self.events.push(("call", target, current_r0));
            *self.returns.get(&target).unwrap_or(&current_r0)
        }
    }

    #[test]
    fn early_local_gates_preserve_incoming_r0() {
        let mut b = B::default();
        assert_eq!(bt_stage76_indirect_orchestrator(0xAA55, &mut b), 0xAA55);
        assert_eq!(b.events, [("enable", STAGE76_BT_ENABLE_FLAGS_ADDR, 0)]);

        let mut b = B { enable: 1, roots: std::vec![0], ..Default::default() };
        assert_eq!(bt_stage76_indirect_orchestrator(7, &mut b), 7);
        assert_eq!(b.events.len(), 2);
    }

    #[test]
    fn nonzero_mode_preflight_can_return_before_provider_side_effects() {
        let mut b = B {
            enable: 1,
            mode_reads: std::vec![1],
            required: [1, 0, 1],
            ..Default::default()
        };
        assert_eq!(bt_stage76_indirect_orchestrator(9, &mut b), 9);
        assert!(!b.events.iter().any(|e| e.0 == "provider"));
        assert!(!b.events.iter().any(|e| e.0 == "request"));
    }

    #[test]
    fn provider_call_is_unconditional_and_zero_return_exits_zero() {
        let mut b = B {
            enable: 1,
            ptrs: BTreeMap::from([((0x2000, 8), 0x3000)]),
            returns: BTreeMap::from([(0x3000, 0)]),
            ..Default::default()
        };
        assert_eq!(bt_stage76_indirect_orchestrator(0x1234, &mut b), 0);
        assert!(b.events.contains(&("request", STAGE76_BT_REQUEST_BYTE_ADDR, 0x34)));
        assert!(b.events.contains(&("call", 0x3000, 0x1234)));
        assert!(!b.events.iter().any(|e| e.0 == "status"));
    }

    #[test]
    fn root_and_mode_are_reread_and_r0_flows_through_zero_mode_chain_and_tail() {
        let roots = std::vec![
            0x1000, // initial nonnull gate
            0x1000, // slot 0
            0x1004, // slot 4
            0x1008, // slot 8
            0x1014, // slot 14
            0x1018, // slot 18
            0x101C, // slot 1c
            0x1020, // slot 20
            0x1024, // slot 24
            0x1028, // tail 28
        ];
        let mut ptrs = BTreeMap::new();
        ptrs.insert((0x2000, 8), 0x3000);
        for (root, off, target) in [
            (0x1000, 0x00, 0x4000),
            (0x1004, 0x04, 0x4004),
            (0x1008, 0x08, 0x4008),
            (0x1014, 0x14, 0x4014),
            (0x1018, 0x18, 0),
            (0x101C, 0x1C, 0x401C),
            (0x1020, 0x20, 0),
            (0x1024, 0x24, 0x4024),
            (0x1028, 0x28, 0x4028),
        ] {
            ptrs.insert((root, off), target);
        }
        let mut returns = BTreeMap::new();
        returns.insert(0x3000, 10);
        returns.insert(0x4000, 11);
        returns.insert(0x4004, 12);
        returns.insert(0x4008, 13);
        returns.insert(0x4014, 14);
        returns.insert(0x401C, 15);
        returns.insert(0x4024, 16);
        returns.insert(0x4028, 17);

        let mut b = B {
            enable: 1,
            mode_reads: std::vec![0, 0],
            roots,
            ptrs,
            returns,
            ..Default::default()
        };
        assert_eq!(bt_stage76_indirect_orchestrator(5, &mut b), 17);
        assert!(b.events.contains(&("status", STAGE76_BT_STATUS_BYTE_ADDR, 2)));
        assert!(b.events.contains(&("call", 0x4000, 10)));
        assert!(b.events.contains(&("call", 0x4004, 11)));
        assert!(b.events.contains(&("call", 0x4008, 12)));
        assert!(b.events.contains(&("call", 0x4014, 13)));
        assert!(b.events.contains(&("call", 0x401C, 14)));
        assert!(b.events.contains(&("call", 0x4024, 15)));
        assert!(b.events.contains(&("call", 0x4028, 16)));
    }

    #[test]
    fn second_mode_read_can_switch_to_nonzero_branch_after_early_callbacks() {
        let roots = std::vec![0x1000,0x1000,0x1000,0x1000,0x1000,0x1000,0x1000];
        let mut ptrs = BTreeMap::new();
        ptrs.insert((0x2000, 8), 0x3000);
        ptrs.insert((0x1000, 0x0C), 0x400C);
        ptrs.insert((0x1000, 0x10), 0x4010);
        let mut returns = BTreeMap::new();
        returns.insert(0x3000, 1);
        returns.insert(0x400C, 2);
        returns.insert(0x4010, 3);
        let mut b = B {
            enable: 1,
            mode_reads: std::vec![0, 1],
            roots,
            ptrs,
            returns,
            ..Default::default()
        };
        assert_eq!(bt_stage76_indirect_orchestrator(9, &mut b), 3);
        assert!(b.events.iter().any(|e| *e == ("ptr", 0x100C, 0x400C)));
        assert!(!b.events.iter().any(|e| e.0 == "ptr" && e.1 == 0x1014));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE76_CURRENT_BT_INDIRECT_ORCHESTRATOR_ADDR, 0x16F970);
        assert_eq!(STAGE76_BT_ENABLE_FLAGS_ADDR, 0x201AF4);
        assert_eq!(STAGE76_BT_CALLBACK_TABLE_ROOT_ADDR, 0x203488);
        assert_eq!(STAGE76_BT_MODE_ADDR, 0x20DAA5);
        assert_eq!(STAGE76_BT_REQUIRED_TABLE_BASE, 0x20DA8C);
        assert_eq!(STAGE76_BT_PROVIDER_ROOT_ADDR, 0x20375C);
        assert_eq!(STAGE76_BT_REQUEST_BYTE_ADDR, 0x20D9A2);
        assert_eq!(STAGE76_BT_STATUS_BYTE_ADDR, 0x20D9A4);
    }
}

/// Stage 77: current fixed-byte post-boundary initializer at `0x170F48`.
///
/// The exact current 30-byte body contains one direct call followed by five
/// byte stores. Masking the single four-byte call encoding leaves 26 fixed
/// bytes and yields one current hit plus one public-legacy structural hit at
/// `0x16D1A0`. Both images resolve the same two literal bases.
pub const STAGE77_CURRENT_BT_FIXED_INIT_ADDR: u32 = 0x0017_0F48;
pub const STAGE77_BT_BOUNDARY: u32 = 0x0000_E558;
pub const STAGE77_BT_BASE_A: u32 = 0x0020_32F2;
pub const STAGE77_BT_BASE_B: u32 = 0x0020_32DC;

pub trait BtStage77Backend {
    /// Current `0xE558` is entered before any R0-R3 mutation in this wrapper.
    fn boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;
    fn write_byte(&mut self, address: u32, value: u8);
}

/// Safe source-level model of current `0x170F48`.
///
/// The opaque boundary receives the four incoming argument registers exactly
/// as they entered the wrapper. Its R0 return remains untouched through all
/// following stores and is therefore the function's final return.
pub fn bt_stage77_fixed_post_boundary_init<B: BtStage77Backend>(
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    backend: &mut B,
) -> u32 {
    let result = backend.boundary(r0, r1, r2, r3);

    backend.write_byte(STAGE77_BT_BASE_A + 1, 1);
    backend.write_byte(STAGE77_BT_BASE_B + 2, 5);
    backend.write_byte(STAGE77_BT_BASE_B + 3, 5);
    backend.write_byte(STAGE77_BT_BASE_B + 0x0B, 0x82);
    backend.write_byte(STAGE77_BT_BASE_B + 0x14, 0xB4);

    result
}

#[cfg(test)]
mod stage77_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        boundary_return: u32,
        args: (u32, u32, u32, u32),
        writes: Vec<(u32, u8)>,
    }

    impl BtStage77Backend for B {
        fn boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32 {
            self.args = (r0, r1, r2, r3);
            self.boundary_return
        }

        fn write_byte(&mut self, address: u32, value: u8) {
            self.writes.push((address, value));
        }
    }

    #[test]
    fn forwards_all_four_argument_registers_and_preserves_boundary_return() {
        let mut b = B { boundary_return: 0xDEAD_BEEF, ..Default::default() };
        assert_eq!(
            bt_stage77_fixed_post_boundary_init(1, 2, 3, 4, &mut b),
            0xDEAD_BEEF
        );
        assert_eq!(b.args, (1, 2, 3, 4));
    }

    #[test]
    fn writes_exact_bytes_in_binary_order() {
        let mut b = B::default();
        let _ = bt_stage77_fixed_post_boundary_init(0, 0, 0, 0, &mut b);
        assert_eq!(b.writes, [
            (0x2032F3, 1),
            (0x2032DE, 5),
            (0x2032DF, 5),
            (0x2032E7, 0x82),
            (0x2032F0, 0xB4),
        ]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE77_CURRENT_BT_FIXED_INIT_ADDR, 0x170F48);
        assert_eq!(STAGE77_BT_BOUNDARY, 0xE558);
        assert_eq!(STAGE77_BT_BASE_A, 0x2032F2);
        assert_eq!(STAGE77_BT_BASE_B, 0x2032DC);
    }
}

/// Stage 78: current 16-bit-progress table fold at `0x1719B8`.
///
/// The exact current 34-byte body is byte-identical to the public-legacy
/// structural counterpart at `0x16DB00`; both bodies are unique in their
/// respective images. The table base itself relocates from legacy `0x2220A8`
/// to current `0x222154`.
pub const STAGE78_CURRENT_BT_TABLE_FOLD_ADDR: u32 = 0x0017_19B8;
pub const STAGE78_BT_TABLE_BASE: u32 = 0x0022_2154;

pub trait BtStage78Backend {
    fn read_byte(&mut self, address: u32) -> u8;
    fn read_table_word(&mut self, address: u32) -> u32;
}

/// Safe source-level model of current `0x1719B8`.
///
/// Firmware computes `progress = UXTH(ptr - start)` on every iteration. That
/// 16-bit truncation is semantically material: lengths above `0xFFFF` never
/// satisfy the unsigned exit comparison, so the exact firmware loop does not
/// terminate for such lengths. This model intentionally preserves that edge.
pub fn bt_stage78_table_fold<B: BtStage78Backend>(
    mut accumulator: u32,
    start: u32,
    length: u32,
    backend: &mut B,
) -> u32 {
    let mut ptr = start;
    loop {
        let progress = ptr.wrapping_sub(start) as u16;
        if length <= u32::from(progress) {
            return accumulator;
        }

        let byte = backend.read_byte(ptr);
        ptr = ptr.wrapping_add(1);

        let index = ((u32::from(byte) ^ accumulator) & 0xFF) as u32;
        let table_word = backend.read_table_word(
            STAGE78_BT_TABLE_BASE.wrapping_add(index.wrapping_mul(4)),
        );
        accumulator = table_word ^ (accumulator >> 8);
    }
}

#[cfg(test)]
mod stage78_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        bytes: Vec<(u32, u8)>,
        table: Vec<(u32, u32)>,
        reads: Vec<(&'static str, u32)>,
    }

    impl BtStage78Backend for B {
        fn read_byte(&mut self, address: u32) -> u8 {
            self.reads.push(("byte", address));
            self.bytes
                .iter()
                .find(|x| x.0 == address)
                .map(|x| x.1)
                .expect("missing byte")
        }

        fn read_table_word(&mut self, address: u32) -> u32 {
            self.reads.push(("table", address));
            self.table
                .iter()
                .find(|x| x.0 == address)
                .map(|x| x.1)
                .expect("missing table word")
        }
    }

    #[test]
    fn zero_length_returns_without_memory_reads() {
        let mut b = B::default();
        assert_eq!(bt_stage78_table_fold(0x1234_5678, 0x2000, 0, &mut b), 0x1234_5678);
        assert!(b.reads.is_empty());
    }

    #[test]
    fn one_byte_uses_low_xor_byte_as_table_index() {
        let acc = 0x1234_5678;
        let byte = 0xA5u8;
        let index = (u32::from(byte) ^ acc) & 0xFF;
        let addr = STAGE78_BT_TABLE_BASE + index * 4;
        let mut b = B {
            bytes: std::vec![(0x1000, byte)],
            table: std::vec![(addr, 0xDEAD_BEEF)],
            ..Default::default()
        };
        assert_eq!(
            bt_stage78_table_fold(acc, 0x1000, 1, &mut b),
            0xDEAD_BEEF ^ (acc >> 8)
        );
        assert_eq!(b.reads, [("byte", 0x1000), ("table", addr)]);
    }

    #[test]
    fn multiple_bytes_preserve_pointer_order_and_accumulator_chaining() {
        let start = 0x3000;
        let first_acc = 0x0000_00AA;
        let b0 = 0x10u8;
        let idx0 = (u32::from(b0) ^ first_acc) & 0xFF;
        let t0 = 0x1122_3344;
        let acc1 = t0 ^ (first_acc >> 8);
        let b1 = 0x20u8;
        let idx1 = (u32::from(b1) ^ acc1) & 0xFF;
        let t1 = 0x5566_7788;
        let mut b = B {
            bytes: std::vec![(start, b0), (start + 1, b1)],
            table: std::vec![
                (STAGE78_BT_TABLE_BASE + idx0 * 4, t0),
                (STAGE78_BT_TABLE_BASE + idx1 * 4, t1),
            ],
            ..Default::default()
        };
        assert_eq!(
            bt_stage78_table_fold(first_acc, start, 2, &mut b),
            t1 ^ (acc1 >> 8)
        );
        assert_eq!(b.reads[0], ("byte", start));
        assert_eq!(b.reads[2], ("byte", start + 1));
    }

    #[test]
    fn progress_is_16_bit_truncated() {
        // Freeze the arithmetic edge without executing the intentionally
        // nonterminating >0xFFFF firmware case.
        let start = 0xFFFF_FFFEu32;
        let ptr = start.wrapping_add(0x1_0001);
        let progress = ptr.wrapping_sub(start) as u16;
        assert_eq!(progress, 1);
        assert!(!(0x1_0000u32 <= u32::from(progress)));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE78_CURRENT_BT_TABLE_FOLD_ADDR, 0x1719B8);
        assert_eq!(STAGE78_BT_TABLE_BASE, 0x222154);
    }
}

/// Stage 79: current snapshot-and-table-fold wrapper at `0x1719E0`.
///
/// The exact current 112-byte body is a relocation-normalized structural
/// counterpart of public-legacy `0x16DB28`. The wrapper snapshots seven
/// external dwords into a fixed buffer, chooses a 32- or 44-byte fold span,
/// delegates the fold to already-recovered Stage 78, then publishes the result.
pub const STAGE79_CURRENT_BT_SNAPSHOT_FOLD_ADDR: u32 = 0x0017_19E0;
pub const STAGE79_BT_PRELUDE_BOUNDARY: u32 = 0x0001_9754;
pub const STAGE79_BT_MEMCPY_BOUNDARY: u32 = 0x0000_3DB4;
pub const STAGE79_BT_STAGE78_ADDR: u32 = STAGE78_CURRENT_BT_TABLE_FOLD_ADDR;
pub const STAGE79_BT_POST_FOLD_BOUNDARY: u32 = 0x0001_9318;
pub const STAGE79_BT_BUFFER_BASE: u32 = 0x0022_2E04;
pub const STAGE79_BT_PREVIOUS_FOLD_ADDR: u32 = 0x0022_2DFC;
pub const STAGE79_BT_INITIALIZED_ADDR: u32 = 0x0022_2DF4;
pub const STAGE79_BT_COPY_SOURCE_PTR_ADDR: u32 = 0x0020_0748;
pub const STAGE79_BT_SOURCE_WORD_ADDRS: [u32; 7] = [
    0x0031_8088,
    0x0032_A004,
    0x0031_86A0,
    0x0041_0434,
    0x0041_079C,
    0x0041_00AC,
    0x0041_0548,
];
pub const STAGE79_BT_SHORT_LENGTH: u32 = 0x20;
pub const STAGE79_BT_LONG_LENGTH: u32 = 0x2C;
pub const STAGE79_BT_INITIAL_ACCUMULATOR: u32 = 0xFFFF_FFFF;

pub trait BtStage79Backend {
    /// Opaque current `0x19754`, called before every snapshot read.
    fn prelude(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;
    fn read_source_word(&mut self, address: u32) -> u32;
    fn write_buffer_word(&mut self, address: u32, value: u32);
    fn read_initialized_word(&mut self) -> u32;
    fn read_previous_fold(&mut self) -> u32;
    fn read_copy_source_ptr(&mut self) -> u32;

    /// Current `0x3DB4(buffer+0x1C, source_ptr-4, 4)`.
    /// Stage 6 already identified this runtime target as the memcpy primitive.
    fn copy_four_bytes(&mut self, destination: u32, source: u32);

    /// Already-recovered current Stage 78 at `0x1719B8`.
    fn stage78_fold(&mut self, accumulator: u32, start: u32, length: u32) -> u32;

    /// Opaque current `0x19318`. Stage 78 returns with R1/R2 preserved and
    /// R3 equal to the terminating 16-bit progress; for local lengths 0x20
    /// and 0x2C that value equals `length`.
    fn post_fold(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;

    fn write_initialized_word(&mut self, value: u32);
    fn write_previous_fold(&mut self, value: u32);
}

/// Safe source-level model of current `0x1719E0`.
///
/// The prelude return and post-fold return are ignored. The final return is
/// always the Stage-78 fold result. When the initialized word is nonzero,
/// the previous fold is copied into buffer +0x1C and only the first 32 bytes
/// are folded. When zero, firmware copies four bytes from
/// `*(0x200748)-4` into buffer +0x1C and folds 44 bytes, intentionally
/// exposing the existing ambient buffer tail +0x20..+0x2B.
pub fn bt_stage79_snapshot_fold<B: BtStage79Backend>(
    incoming_r0: u32,
    incoming_r1: u32,
    incoming_r2: u32,
    incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let _ = backend.prelude(incoming_r0, incoming_r1, incoming_r2, incoming_r3);

    let mut index = 0usize;
    while index < STAGE79_BT_SOURCE_WORD_ADDRS.len() {
        let value = backend.read_source_word(STAGE79_BT_SOURCE_WORD_ADDRS[index]);
        backend.write_buffer_word(
            STAGE79_BT_BUFFER_BASE.wrapping_add((index as u32).wrapping_mul(4)),
            value,
        );
        index += 1;
    }

    let length = if backend.read_initialized_word() != 0 {
        let previous = backend.read_previous_fold();
        backend.write_buffer_word(STAGE79_BT_BUFFER_BASE + 0x1C, previous);
        STAGE79_BT_SHORT_LENGTH
    } else {
        let source = backend.read_copy_source_ptr().wrapping_sub(4);
        backend.copy_four_bytes(STAGE79_BT_BUFFER_BASE + 0x1C, source);
        STAGE79_BT_LONG_LENGTH
    };

    let fold = backend.stage78_fold(
        STAGE79_BT_INITIAL_ACCUMULATOR,
        STAGE79_BT_BUFFER_BASE,
        length,
    );

    let _ = backend.post_fold(fold, STAGE79_BT_BUFFER_BASE, length, length);
    backend.write_initialized_word(1);
    backend.write_previous_fold(fold);
    fold
}

#[cfg(test)]
mod stage79_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        source_words: [u32; 7],
        initialized: u32,
        previous: u32,
        copy_source_ptr: u32,
        fold_return: u32,
        post_return: u32,
        events: Vec<(&'static str, u32, u32, u32, u32)>,
    }

    impl BtStage79Backend for B {
        fn prelude(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->u32 {
            self.events.push(("prelude",r0,r1,r2,r3)); 0xAAAA_AAAA
        }
        fn read_source_word(&mut self,address:u32)->u32 {
            self.events.push(("read_word",address,0,0,0));
            let i=STAGE79_BT_SOURCE_WORD_ADDRS.iter().position(|&x|x==address).unwrap();
            self.source_words[i]
        }
        fn write_buffer_word(&mut self,address:u32,value:u32) {
            self.events.push(("write_buffer",address,value,0,0));
        }
        fn read_initialized_word(&mut self)->u32 {
            self.events.push(("read_init",STAGE79_BT_INITIALIZED_ADDR,0,0,0));
            self.initialized
        }
        fn read_previous_fold(&mut self)->u32 {
            self.events.push(("read_prev",STAGE79_BT_PREVIOUS_FOLD_ADDR,0,0,0));
            self.previous
        }
        fn read_copy_source_ptr(&mut self)->u32 {
            self.events.push(("read_copy_ptr",STAGE79_BT_COPY_SOURCE_PTR_ADDR,0,0,0));
            self.copy_source_ptr
        }
        fn copy_four_bytes(&mut self,destination:u32,source:u32) {
            self.events.push(("copy4",destination,source,4,0));
        }
        fn stage78_fold(&mut self,accumulator:u32,start:u32,length:u32)->u32 {
            self.events.push(("stage78",accumulator,start,length,0));
            self.fold_return
        }
        fn post_fold(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->u32 {
            self.events.push(("post_fold",r0,r1,r2,r3));
            self.post_return
        }
        fn write_initialized_word(&mut self,value:u32) {
            self.events.push(("write_init",STAGE79_BT_INITIALIZED_ADDR,value,0,0));
            self.initialized=value;
        }
        fn write_previous_fold(&mut self,value:u32) {
            self.events.push(("write_prev",STAGE79_BT_PREVIOUS_FOLD_ADDR,value,0,0));
            self.previous=value;
        }
    }

    #[test]
    fn initialized_path_snapshots_after_prelude_then_folds_32_bytes() {
        let mut b=B {
            source_words:[1,2,3,4,5,6,7],
            initialized:1,
            previous:0x1122_3344,
            fold_return:0xDEAD_BEEF,
            post_return:0xAAAA_5555,
            ..Default::default()
        };
        assert_eq!(bt_stage79_snapshot_fold(10,11,12,13,&mut b),0xDEAD_BEEF);
        assert_eq!(b.events[0],("prelude",10,11,12,13));
        for i in 0..7 {
            assert_eq!(b.events[1+i*2],("read_word",STAGE79_BT_SOURCE_WORD_ADDRS[i],0,0,0));
            assert_eq!(b.events[2+i*2],("write_buffer",STAGE79_BT_BUFFER_BASE+(i as u32)*4,(i+1) as u32,0,0));
        }
        assert!(b.events.iter().any(|e| *e==("write_buffer",STAGE79_BT_BUFFER_BASE+0x1C,0x1122_3344,0,0)));
        assert!(b.events.iter().any(|e| *e==("stage78",0xFFFF_FFFF,STAGE79_BT_BUFFER_BASE,0x20,0)));
        assert!(b.events.iter().any(|e| *e==("post_fold",0xDEAD_BEEF,STAGE79_BT_BUFFER_BASE,0x20,0x20)));
        assert_eq!(b.previous,0xDEAD_BEEF);
        assert_eq!(b.initialized,1);
    }

    #[test]
    fn zero_initialized_path_copies_source_minus_four_and_folds_44_bytes() {
        let mut b=B {
            initialized:0,
            copy_source_ptr:2,
            fold_return:0x0102_0304,
            ..Default::default()
        };
        assert_eq!(bt_stage79_snapshot_fold(0,0,0,0,&mut b),0x0102_0304);
        assert!(b.events.iter().any(|e| *e==(
            "copy4",
            STAGE79_BT_BUFFER_BASE+0x1C,
            0xFFFF_FFFE,
            4,
            0
        )));
        assert!(b.events.iter().any(|e| *e==(
            "stage78",
            0xFFFF_FFFF,
            STAGE79_BT_BUFFER_BASE,
            0x2C,
            0
        )));
        assert!(b.events.iter().any(|e| *e==(
            "post_fold",
            0x0102_0304,
            STAGE79_BT_BUFFER_BASE,
            0x2C,
            0x2C
        )));
    }

    #[test]
    fn post_fold_return_is_ignored_and_publish_order_is_exact() {
        let mut b=B {
            initialized:1,
            fold_return:0x1234_5678,
            post_return:0xFFFF_FFFF,
            ..Default::default()
        };
        assert_eq!(bt_stage79_snapshot_fold(0,0,0,0,&mut b),0x1234_5678);
        let p=b.events.iter().position(|e|e.0=="post_fold").unwrap();
        assert_eq!(b.events[p+1],("write_init",STAGE79_BT_INITIALIZED_ADDR,1,0,0));
        assert_eq!(b.events[p+2],("write_prev",STAGE79_BT_PREVIOUS_FOLD_ADDR,0x1234_5678,0,0));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE79_CURRENT_BT_SNAPSHOT_FOLD_ADDR,0x1719E0);
        assert_eq!(STAGE79_BT_PRELUDE_BOUNDARY,0x19754);
        assert_eq!(STAGE79_BT_MEMCPY_BOUNDARY,0x3DB4);
        assert_eq!(STAGE79_BT_STAGE78_ADDR,0x1719B8);
        assert_eq!(STAGE79_BT_POST_FOLD_BOUNDARY,0x19318);
        assert_eq!(STAGE79_BT_BUFFER_BASE,0x222E04);
        assert_eq!(STAGE79_BT_PREVIOUS_FOLD_ADDR,0x222DFC);
        assert_eq!(STAGE79_BT_INITIALIZED_ADDR,0x222DF4);
        assert_eq!(STAGE79_BT_COPY_SOURCE_PTR_ADDR,0x200748);
    }
}

/// Stage 80: current fixed global initializer at `0x171B5C`.
///
/// The exact current 24-byte leaf is unique in the current HCD. No public
/// legacy structural counterpart is promoted, including after masking only
/// the four PC-relative literal imm8 bytes.
pub const STAGE80_CURRENT_BT_FIXED_GLOBAL_INIT_ADDR: u32 = 0x0017_1B5C;
pub const STAGE80_BT_POINTER_SLOT_A_ADDR: u32 = 0x0020_4B18;
pub const STAGE80_BT_POINTER_SLOT_B_ADDR: u32 = 0x0020_4B10;
pub const STAGE80_BT_SHARED_POINTER_VALUE: u32 = 0x0003_D090;
pub const STAGE80_BT_STATE_BASE_ADDR: u32 = 0x0035_2600;
pub const STAGE80_BT_STATE_WORD14_ADDR: u32 = STAGE80_BT_STATE_BASE_ADDR + 0x14;
pub const STAGE80_BT_STATE_WORD14_VALUE: u32 = 0x0000_1FFF;

pub trait BtStage80Backend {
    fn write32(&mut self, address: u32, value: u32);
}

/// Safe source-level model of current `0x171B5C`.
///
/// Firmware performs four writes in this exact order and never modifies R0,
/// so the incoming R0 value is the final return.
pub fn bt_stage80_fixed_global_init<B: BtStage80Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    backend.write32(STAGE80_BT_POINTER_SLOT_A_ADDR, STAGE80_BT_SHARED_POINTER_VALUE);
    backend.write32(STAGE80_BT_POINTER_SLOT_B_ADDR, STAGE80_BT_SHARED_POINTER_VALUE);
    backend.write32(STAGE80_BT_STATE_BASE_ADDR, 0);
    backend.write32(STAGE80_BT_STATE_WORD14_ADDR, STAGE80_BT_STATE_WORD14_VALUE);
    incoming_r0
}

#[cfg(test)]
mod stage80_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        writes: Vec<(u32, u32)>,
    }

    impl BtStage80Backend for B {
        fn write32(&mut self, address: u32, value: u32) {
            self.writes.push((address, value));
        }
    }

    #[test]
    fn writes_exact_values_in_binary_order() {
        let mut b = B::default();
        assert_eq!(bt_stage80_fixed_global_init(0xAABB_CCDD, &mut b), 0xAABB_CCDD);
        assert_eq!(b.writes, [
            (STAGE80_BT_POINTER_SLOT_A_ADDR, STAGE80_BT_SHARED_POINTER_VALUE),
            (STAGE80_BT_POINTER_SLOT_B_ADDR, STAGE80_BT_SHARED_POINTER_VALUE),
            (STAGE80_BT_STATE_BASE_ADDR, 0),
            (STAGE80_BT_STATE_WORD14_ADDR, STAGE80_BT_STATE_WORD14_VALUE),
        ]);
    }

    #[test]
    fn return_is_incoming_r0_even_for_zero() {
        let mut b = B::default();
        assert_eq!(bt_stage80_fixed_global_init(0, &mut b), 0);
        assert_eq!(b.writes.len(), 4);
    }

    #[test]
    fn provenance_constants_are_current_only() {
        assert_eq!(STAGE80_CURRENT_BT_FIXED_GLOBAL_INIT_ADDR, 0x171B5C);
        assert_eq!(STAGE80_BT_POINTER_SLOT_A_ADDR, 0x204B18);
        assert_eq!(STAGE80_BT_POINTER_SLOT_B_ADDR, 0x204B10);
        assert_eq!(STAGE80_BT_SHARED_POINTER_VALUE, 0x3D090);
        assert_eq!(STAGE80_BT_STATE_BASE_ADDR, 0x352600);
        assert_eq!(STAGE80_BT_STATE_WORD14_ADDR, 0x352614);
        assert_eq!(STAGE80_BT_STATE_WORD14_VALUE, 0x1FFF);
    }
}

/// Stage 81: current critical-state repair wrapper at `0x171B84`.
///
/// The exact current 62-byte body contains two calls/tail-transfers to the already
/// identified critical-state swap boundary at `0x780` and one opaque call to
/// `0x15180`. No public-legacy structural counterpart is promoted.
pub const STAGE81_CURRENT_BT_CRITICAL_REPAIR_ADDR: u32 = 0x0017_1B84;
pub const STAGE81_BT_CRITICAL_BOUNDARY: u32 = ROM_CRITICAL_STATE_SWAP_LIKE_ADDR;
pub const STAGE81_BT_EXPECTED_WORD_ADDR: u32 = 0x0035_2614;
pub const STAGE81_BT_EXPECTED_WORD_VALUE: u32 = 0x0000_1FFF;
pub const STAGE81_BT_RESET_WORD_ADDR: u32 = 0x0035_2600;
pub const STAGE81_BT_FLAG_ADDR: u32 = 0x0021_70EF;
pub const STAGE81_BT_FINALIZE_CONTEXT_ADDR: u32 = 0x0021_7174;
pub const STAGE81_BT_FINALIZE_BOUNDARY: u32 = 0x0001_5180;

pub trait BtStage81Backend {
    /// Current `0x780(value)`. The first call receives literal one; the final
    /// tail call receives the token returned by the first call.
    fn critical_swap(&mut self, value: u32) -> u32;
    fn read_word(&mut self, address: u32) -> u32;
    fn write_word(&mut self, address: u32, value: u32);
    fn read_flag_byte(&mut self, address: u32) -> u8;
    fn write_flag_byte(&mut self, address: u32, value: u8);

    /// Current opaque `0x15180(0x217174, incoming_r0, 1)` call shape.
    /// Its return is ignored locally.
    fn finalize_boundary(&mut self, context: u32, incoming_r0: u32, literal_one: u32) -> u32;
}

/// Safe source-level model of current `0x171B84`.
///
/// Firmware enters the critical-state boundary with literal one and preserves
/// the returned token. It repairs the two fixed dwords only when the expected
/// word differs from `0x1FFF`. It then performs a one-time flag transition and
/// optional opaque boundary call. Finally it tail-restores the critical state
/// with the saved token; that restore return is the wrapper's final return.
pub fn bt_stage81_critical_repair<B: BtStage81Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    let token = backend.critical_swap(1);

    if backend.read_word(STAGE81_BT_EXPECTED_WORD_ADDR) != STAGE81_BT_EXPECTED_WORD_VALUE {
        backend.write_word(STAGE81_BT_RESET_WORD_ADDR, 0);
        backend.write_word(STAGE81_BT_EXPECTED_WORD_ADDR, STAGE81_BT_EXPECTED_WORD_VALUE);
    }

    if backend.read_flag_byte(STAGE81_BT_FLAG_ADDR) == 0 {
        backend.write_flag_byte(STAGE81_BT_FLAG_ADDR, 1);
        let _ = backend.finalize_boundary(STAGE81_BT_FINALIZE_CONTEXT_ADDR, incoming_r0, 1);
    }

    backend.critical_swap(token)
}

#[cfg(test)]
mod stage81_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        expected: u32,
        flag: u8,
        enter_token: u32,
        restore_return: u32,
        finalize_return: u32,
        events: Vec<(&'static str, u32, u32, u32)>,
    }

    impl BtStage81Backend for B {
        fn critical_swap(&mut self, value: u32) -> u32 {
            self.events.push(("critical", value, 0, 0));
            if value == 1 { self.enter_token } else { self.restore_return }
        }
        fn read_word(&mut self, address: u32) -> u32 {
            self.events.push(("read_word", address, 0, 0));
            self.expected
        }
        fn write_word(&mut self, address: u32, value: u32) {
            self.events.push(("write_word", address, value, 0));
            if address == STAGE81_BT_EXPECTED_WORD_ADDR { self.expected = value; }
        }
        fn read_flag_byte(&mut self, address: u32) -> u8 {
            self.events.push(("read_flag", address, 0, 0));
            self.flag
        }
        fn write_flag_byte(&mut self, address: u32, value: u8) {
            self.events.push(("write_flag", address, u32::from(value), 0));
            self.flag = value;
        }
        fn finalize_boundary(&mut self, context: u32, incoming_r0: u32, literal_one: u32) -> u32 {
            self.events.push(("finalize", context, incoming_r0, literal_one));
            self.finalize_return
        }
    }

    #[test]
    fn matching_word_and_set_flag_only_enter_and_restore_critical_state() {
        let mut b = B {
            expected: STAGE81_BT_EXPECTED_WORD_VALUE,
            flag: 1,
            enter_token: 0x55,
            restore_return: 0xABCD,
            ..Default::default()
        };
        assert_eq!(bt_stage81_critical_repair(0x1234, &mut b), 0xABCD);
        assert_eq!(b.events, [
            ("critical", 1, 0, 0),
            ("read_word", STAGE81_BT_EXPECTED_WORD_ADDR, 0, 0),
            ("read_flag", STAGE81_BT_FLAG_ADDR, 0, 0),
            ("critical", 0x55, 0, 0),
        ]);
    }

    #[test]
    fn mismatch_and_zero_flag_preserve_repair_and_finalize_order() {
        let mut b = B {
            expected: 7,
            flag: 0,
            enter_token: 0xCAFE,
            restore_return: 0xDEAD_BEEF,
            finalize_return: 0x1111,
            ..Default::default()
        };
        assert_eq!(bt_stage81_critical_repair(0x2233_4455, &mut b), 0xDEAD_BEEF);
        assert_eq!(b.expected, STAGE81_BT_EXPECTED_WORD_VALUE);
        assert_eq!(b.flag, 1);
        assert_eq!(b.events, [
            ("critical", 1, 0, 0),
            ("read_word", STAGE81_BT_EXPECTED_WORD_ADDR, 0, 0),
            ("write_word", STAGE81_BT_RESET_WORD_ADDR, 0, 0),
            ("write_word", STAGE81_BT_EXPECTED_WORD_ADDR, STAGE81_BT_EXPECTED_WORD_VALUE, 0),
            ("read_flag", STAGE81_BT_FLAG_ADDR, 0, 0),
            ("write_flag", STAGE81_BT_FLAG_ADDR, 1, 0),
            ("finalize", STAGE81_BT_FINALIZE_CONTEXT_ADDR, 0x2233_4455, 1),
            ("critical", 0xCAFE, 0, 0),
        ]);
    }

    #[test]
    fn finalize_return_is_ignored_and_restore_return_is_final() {
        let mut b = B {
            expected: STAGE81_BT_EXPECTED_WORD_VALUE,
            flag: 0,
            enter_token: 9,
            restore_return: 77,
            finalize_return: u32::MAX,
            ..Default::default()
        };
        assert_eq!(bt_stage81_critical_repair(0, &mut b), 77);
    }

    #[test]
    fn provenance_constants_are_current_only() {
        assert_eq!(STAGE81_CURRENT_BT_CRITICAL_REPAIR_ADDR, 0x171B84);
        assert_eq!(STAGE81_BT_CRITICAL_BOUNDARY, 0x780);
        assert_eq!(STAGE81_BT_EXPECTED_WORD_ADDR, 0x352614);
        assert_eq!(STAGE81_BT_EXPECTED_WORD_VALUE, 0x1FFF);
        assert_eq!(STAGE81_BT_RESET_WORD_ADDR, 0x352600);
        assert_eq!(STAGE81_BT_FLAG_ADDR, 0x2170EF);
        assert_eq!(STAGE81_BT_FINALIZE_CONTEXT_ADDR, 0x217174);
        assert_eq!(STAGE81_BT_FINALIZE_BOUNDARY, 0x15180);
    }
}

/// Stage 82: current reset/gate/dispatch sequence at `0x171BD4`.
///
/// The exact 112-byte current body has one public-legacy structural counterpart at
/// `0x16D98C`. Masking seven four-byte direct-call encodings leaves 84 fixed bytes.
/// The two setup boundaries and the two late chain boundaries remain opaque; the
/// already recovered Stage-81 call is represented explicitly as a boundary here so
/// its possible ambient-memory effects remain visible to the mandatory status reread.
pub const STAGE82_CURRENT_BT_RESET_GATE_SEQUENCE_ADDR: u32 = 0x0017_1BD4;
pub const STAGE82_BT_SETUP_ZERO_BOUNDARY: u32 = 0x0001_51E0;
pub const STAGE82_BT_SETUP_ONE_BOUNDARY: u32 = 0x0001_5214;
pub const STAGE82_BT_SETUP_CONTEXT_ADDR: u32 = 0x0021_7174;
pub const STAGE82_BT_SETUP_CALLBACK_THUMB: u32 = 0x0017_1D05;
pub const STAGE82_BT_RESET_BYTE_ADDR: u32 = 0x0021_70EF;
pub const STAGE82_BT_RESET_HALFWORD_A_ADDR: u32 = 0x0021_70EC;
pub const STAGE82_BT_RESET_HALFWORD_B_ADDR: u32 = 0x0021_7170;
pub const STAGE82_BT_HALFWORD_50_ADDR: u32 = 0x0020_4B14;
pub const STAGE82_BT_STATUS_WORD_ADDR: u32 = 0x0035_2604;
pub const STAGE82_BT_LOW20_EXPECTED: u32 = 0x000F_FFFF;
pub const STAGE82_BT_MATCH_FLAG_ADDR: u32 = 0x0021_70EE;
pub const STAGE82_BT_STAGE81_INPUT_ADDR: u32 = 0x0020_4B18;
pub const STAGE82_BT_STAGE81_BOUNDARY: u32 = STAGE81_CURRENT_BT_CRITICAL_REPAIR_ADDR;
pub const STAGE82_BT_FULL_EXPECTED: u32 = 0x200F_FFFF;
pub const STAGE82_BT_CHAIN_INPUT_ADDR: u32 = 0x0035_2608;
pub const STAGE82_BT_CHAIN_FIRST_BOUNDARY: u32 = 0x000B_AA08;
pub const STAGE82_BT_CHAIN_REPEAT_BOUNDARY: u32 = 0x000B_A988;
pub const STAGE82_BT_RESULT_WORD_ADDR: u32 = 0x0022_2E00;

pub trait BtStage82Backend {
    fn setup_zero(&mut self, context: u32, callback_thumb: u32, zero: u32) -> u32;
    fn setup_one(&mut self, context: u32, one: u32) -> u32;

    fn write_byte(&mut self, address: u32, value: u8);
    fn write_halfword(&mut self, address: u32, value: u16);
    fn read_word(&mut self, address: u32) -> u32;
    fn write_word(&mut self, address: u32, value: u32);

    /// Current direct call to recovered Stage 81. Its return becomes current R0.
    /// Implementations may also mutate ambient state, which is why firmware rereads
    /// `STAGE82_BT_STATUS_WORD_ADDR` after this call.
    fn stage81_boundary(&mut self, value: u32) -> u32;

    fn chain_first(&mut self, value: u32) -> u32;
    fn chain_repeat(&mut self, value: u32) -> u32;
}

/// Safe source-level model of current `0x171BD4`.
///
/// The return from setup-zero is ignored. The setup-one return stays live unless
/// replaced by Stage 81 or by the late four-call chain. The status word is read once
/// for the low-20-bit gate and read again after the optional Stage-81 call for the
/// full-word equality gate. The final result dword is always written as zero or one,
/// and that store does not alter the current R0 return value.
pub fn bt_stage82_reset_gate_sequence<B: BtStage82Backend>(
    backend: &mut B,
) -> u32 {
    let _ = backend.setup_zero(STAGE82_BT_SETUP_CONTEXT_ADDR, STAGE82_BT_SETUP_CALLBACK_THUMB, 0);
    let mut current_r0 = backend.setup_one(STAGE82_BT_SETUP_CONTEXT_ADDR, 1);

    backend.write_byte(STAGE82_BT_RESET_BYTE_ADDR, 0);
    backend.write_halfword(STAGE82_BT_RESET_HALFWORD_A_ADDR, 0);
    backend.write_halfword(STAGE82_BT_RESET_HALFWORD_B_ADDR, 0);
    backend.write_halfword(STAGE82_BT_HALFWORD_50_ADDR, 0x50);

    let first_status = backend.read_word(STAGE82_BT_STATUS_WORD_ADDR);
    if (first_status & STAGE82_BT_LOW20_EXPECTED) == STAGE82_BT_LOW20_EXPECTED {
        backend.write_byte(STAGE82_BT_MATCH_FLAG_ADDR, 1);
    } else {
        let value = backend.read_word(STAGE82_BT_STAGE81_INPUT_ADDR);
        current_r0 = backend.stage81_boundary(value);
    }

    let final_status = backend.read_word(STAGE82_BT_STATUS_WORD_ADDR);
    let matched = final_status == STAGE82_BT_FULL_EXPECTED;
    if matched {
        let value = backend.read_word(STAGE82_BT_CHAIN_INPUT_ADDR);
        current_r0 = backend.chain_first(value);
        current_r0 = backend.chain_repeat(current_r0);
        current_r0 = backend.chain_repeat(current_r0);
        current_r0 = backend.chain_repeat(current_r0);
    }

    backend.write_word(STAGE82_BT_RESULT_WORD_ADDR, if matched { 1 } else { 0 });
    current_r0
}

#[cfg(test)]
mod stage82_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        status: u32,
        stage81_status_after: Option<u32>,
        stage81_input: u32,
        chain_input: u32,
        setup_one_return: u32,
        stage81_return: u32,
        chain_returns: [u32; 4],
        chain_index: usize,
        events: Vec<(&'static str, u32, u32, u32)>,
    }

    impl BtStage82Backend for B {
        fn setup_zero(&mut self, context: u32, callback_thumb: u32, zero: u32) -> u32 {
            self.events.push(("setup_zero", context, callback_thumb, zero));
            0xAAAA_AAAA
        }
        fn setup_one(&mut self, context: u32, one: u32) -> u32 {
            self.events.push(("setup_one", context, one, 0));
            self.setup_one_return
        }
        fn write_byte(&mut self, address: u32, value: u8) {
            self.events.push(("write_byte", address, u32::from(value), 0));
        }
        fn write_halfword(&mut self, address: u32, value: u16) {
            self.events.push(("write_halfword", address, u32::from(value), 0));
        }
        fn read_word(&mut self, address: u32) -> u32 {
            self.events.push(("read_word", address, 0, 0));
            match address {
                STAGE82_BT_STATUS_WORD_ADDR => self.status,
                STAGE82_BT_STAGE81_INPUT_ADDR => self.stage81_input,
                STAGE82_BT_CHAIN_INPUT_ADDR => self.chain_input,
                _ => 0,
            }
        }
        fn write_word(&mut self, address: u32, value: u32) {
            self.events.push(("write_word", address, value, 0));
        }
        fn stage81_boundary(&mut self, value: u32) -> u32 {
            self.events.push(("stage81", value, 0, 0));
            if let Some(v) = self.stage81_status_after { self.status = v; }
            self.stage81_return
        }
        fn chain_first(&mut self, value: u32) -> u32 {
            self.events.push(("chain_first", value, 0, 0));
            self.chain_index = 1;
            self.chain_returns[0]
        }
        fn chain_repeat(&mut self, value: u32) -> u32 {
            self.events.push(("chain_repeat", value, 0, 0));
            let out = self.chain_returns[self.chain_index];
            self.chain_index += 1;
            out
        }
    }

    #[test]
    fn low20_match_sets_flag_and_nonfull_status_preserves_setup_return() {
        let mut b = B {
            status: STAGE82_BT_LOW20_EXPECTED,
            setup_one_return: 0x1234,
            ..Default::default()
        };
        assert_eq!(bt_stage82_reset_gate_sequence(&mut b), 0x1234);
        assert!(b.events.contains(&("write_byte", STAGE82_BT_MATCH_FLAG_ADDR, 1, 0)));
        assert!(!b.events.iter().any(|x| x.0 == "stage81"));
        assert_eq!(b.events.last(), Some(&("write_word", STAGE82_BT_RESULT_WORD_ADDR, 0, 0)));
    }

    #[test]
    fn stage81_mutation_is_visible_to_full_status_reread_and_chain_threads_r0() {
        let mut b = B {
            status: 0,
            stage81_status_after: Some(STAGE82_BT_FULL_EXPECTED),
            stage81_input: 0x55,
            stage81_return: 0x100,
            chain_input: 0x77,
            chain_returns: [0x10, 0x20, 0x30, 0x40],
            ..Default::default()
        };
        assert_eq!(bt_stage82_reset_gate_sequence(&mut b), 0x40);
        assert_eq!(b.events.iter().filter(|x| x.0 == "read_word" && x.1 == STAGE82_BT_STATUS_WORD_ADDR).count(), 2);
        assert_eq!(b.events.iter().filter(|x| x.0 == "chain_repeat").map(|x| x.1).collect::<Vec<_>>(), [0x10, 0x20, 0x30]);
        assert_eq!(b.events.last(), Some(&("write_word", STAGE82_BT_RESULT_WORD_ADDR, 1, 0)));
    }

    #[test]
    fn full_match_without_stage81_runs_chain_and_overrides_setup_return() {
        let mut b = B {
            status: STAGE82_BT_FULL_EXPECTED,
            setup_one_return: 0x9999,
            chain_input: 5,
            chain_returns: [6, 7, 8, 9],
            ..Default::default()
        };
        assert_eq!(bt_stage82_reset_gate_sequence(&mut b), 9);
        assert!(!b.events.iter().any(|x| x.0 == "stage81"));
        assert!(b.events.contains(&("write_byte", STAGE82_BT_MATCH_FLAG_ADDR, 1, 0)));
    }

    #[test]
    fn reset_writes_precede_status_gate_in_binary_order() {
        let mut b = B { status: STAGE82_BT_LOW20_EXPECTED, ..Default::default() };
        let _ = bt_stage82_reset_gate_sequence(&mut b);
        let names = b.events.iter().map(|x| x.0).collect::<Vec<_>>();
        assert_eq!(&names[..7], [
            "setup_zero", "setup_one",
            "write_byte", "write_halfword", "write_halfword", "write_halfword",
            "read_word",
        ]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE82_CURRENT_BT_RESET_GATE_SEQUENCE_ADDR, 0x171BD4);
        assert_eq!(STAGE82_BT_STAGE81_BOUNDARY, 0x171B84);
        assert_eq!(STAGE82_BT_STATUS_WORD_ADDR, 0x352604);
        assert_eq!(STAGE82_BT_LOW20_EXPECTED, 0xFFFFF);
        assert_eq!(STAGE82_BT_FULL_EXPECTED, 0x200FFFFF);
        assert_eq!(STAGE82_BT_RESULT_WORD_ADDR, 0x222E00);
        assert_eq!(STAGE82_BT_CHAIN_FIRST_BOUNDARY, 0xBAA08);
        assert_eq!(STAGE82_BT_CHAIN_REPEAT_BOUNDARY, 0xBA988);
    }
}

/// Stage 83: current low-20/full-word gate wrapper at `0x171D08`.
///
/// The exact current 62-byte body has one public-legacy structural counterpart at
/// `0x16D92C`. Two four-byte control-transfer encodings are relocation-masked.
/// The input word addressed by incoming R2 is deliberately read twice on the
/// match path: the first snapshot supplies only low 20 bits for comparison with
/// full incoming R1, while the second snapshot is compared against exact
/// `0x200FFFFF`.
pub const STAGE83_CURRENT_BT_LOW20_GATE_ADDR: u32 = 0x0017_1D08;
pub const STAGE83_BT_SPECIAL_BOUNDARY: u32 = 0x000B_AA08;
pub const STAGE83_BT_STAGE81_TAIL_ADDR: u32 = STAGE81_CURRENT_BT_CRITICAL_REPAIR_ADDR;
pub const STAGE83_BT_EXACT_WORD: u32 = 0x200F_FFFF;
pub const STAGE83_BT_LOW20_MASK: u32 = 0x000F_FFFF;
pub const STAGE83_BT_SPECIAL_INPUT_ADDR: u32 = 0x0035_2608;
pub const STAGE83_BT_PUBLISH_WORD_ADDR: u32 = 0x0022_2E00;
pub const STAGE83_BT_READY_FLAG_ADDR: u32 = 0x0021_70EE;
pub const STAGE83_BT_REJECT_FLAG_ADDR: u32 = 0x0021_70EF;
pub const STAGE83_BT_REPAIR_INPUT_ADDR: u32 = 0x0020_4B10;

pub trait BtStage83Backend {
    /// Loads the dword pointed to by incoming R2. Firmware can perform this twice.
    fn read_input_word(&mut self, input_ptr: u32) -> u32;
    fn read_word(&mut self, address: u32) -> u32;
    fn write_word(&mut self, address: u32, value: u32);
    fn write_byte(&mut self, address: u32, value: u8);

    /// Current opaque `0xBAA08(value)`.
    fn special_boundary(&mut self, value: u32) -> u32;

    /// Already recovered Stage-81 tail at current `0x171B84`.
    fn stage81_tail(&mut self, value: u32) -> u32;
}

/// Safe source-level model of current `0x171D08`.
///
/// On low-20 mismatch the local wrapper clears `0x2170EF`, reads fixed repair
/// input `0x204B10`, logical-shifts it right by one, and tail-forwards that value
/// into Stage 81. On low-20 match firmware rereads the full input dword. Exact
/// `0x200FFFFF` invokes `0xBAA08(*0x352608)` and writes one to dword `0x222E00`;
/// either way the match path then writes one to byte `0x2170EE`. The special
/// boundary return remains final through those stores; a non-special match
/// returns incoming R0 unchanged.
pub fn bt_stage83_low20_fullword_gate<B: BtStage83Backend>(
    incoming_r0: u32,
    incoming_r1: u32,
    input_ptr: u32,
    backend: &mut B,
) -> u32 {
    let first = backend.read_input_word(input_ptr);
    if (first & STAGE83_BT_LOW20_MASK) != incoming_r1 {
        backend.write_byte(STAGE83_BT_REJECT_FLAG_ADDR, 0);
        let repair = backend.read_word(STAGE83_BT_REPAIR_INPUT_ADDR);
        return backend.stage81_tail(repair >> 1);
    }

    let full = backend.read_input_word(input_ptr);
    let result = if full == STAGE83_BT_EXACT_WORD {
        let value = backend.read_word(STAGE83_BT_SPECIAL_INPUT_ADDR);
        let result = backend.special_boundary(value);
        backend.write_word(STAGE83_BT_PUBLISH_WORD_ADDR, 1);
        result
    } else {
        incoming_r0
    };

    backend.write_byte(STAGE83_BT_READY_FLAG_ADDR, 1);
    result
}

#[cfg(test)]
mod stage83_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        input_reads: Vec<u32>,
        input_values: Vec<u32>,
        repair: u32,
        special_input: u32,
        special_return: u32,
        stage81_return: u32,
        events: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage83Backend for B {
        fn read_input_word(&mut self, input_ptr: u32) -> u32 {
            self.events.push(("read_input", input_ptr, 0));
            let value = self.input_values[self.input_reads.len()];
            self.input_reads.push(value);
            value
        }
        fn read_word(&mut self, address: u32) -> u32 {
            self.events.push(("read_word", address, 0));
            if address == STAGE83_BT_REPAIR_INPUT_ADDR { self.repair } else { self.special_input }
        }
        fn write_word(&mut self, address: u32, value: u32) {
            self.events.push(("write_word", address, value));
        }
        fn write_byte(&mut self, address: u32, value: u8) {
            self.events.push(("write_byte", address, u32::from(value)));
        }
        fn special_boundary(&mut self, value: u32) -> u32 {
            self.events.push(("special", value, 0));
            self.special_return
        }
        fn stage81_tail(&mut self, value: u32) -> u32 {
            self.events.push(("stage81", value, 0));
            self.stage81_return
        }
    }

    #[test]
    fn low20_mismatch_is_one_read_then_clear_shift_and_stage81_tail() {
        let mut b = B { input_values: std::vec![0x1234_5678], repair: 0x8000_0003, stage81_return: 77, ..Default::default() };
        assert_eq!(bt_stage83_low20_fullword_gate(9, 0x45679, 0x1000, &mut b), 77);
        assert_eq!(b.input_reads, [0x1234_5678]);
        assert_eq!(b.events, [
            ("read_input", 0x1000, 0),
            ("write_byte", STAGE83_BT_REJECT_FLAG_ADDR, 0),
            ("read_word", STAGE83_BT_REPAIR_INPUT_ADDR, 0),
            ("stage81", 0x4000_0001, 0),
        ]);
    }

    #[test]
    fn matching_low20_rereads_full_word_and_non_special_returns_incoming_r0() {
        let mut b = B { input_values: std::vec![0xABCF_FFFF, 0x111F_FFFF], ..Default::default() };
        assert_eq!(bt_stage83_low20_fullword_gate(0xCAFE, 0xF_FFFF, 0x2000, &mut b), 0xCAFE);
        assert_eq!(b.input_reads, [0xABCF_FFFF, 0x111F_FFFF]);
        assert_eq!(b.events, [
            ("read_input", 0x2000, 0),
            ("read_input", 0x2000, 0),
            ("write_byte", STAGE83_BT_READY_FLAG_ADDR, 1),
        ]);
    }

    #[test]
    fn exact_second_read_calls_special_publishes_then_sets_ready_and_preserves_return() {
        let mut b = B {
            input_values: std::vec![0xAA0F_FFFF, STAGE83_BT_EXACT_WORD],
            special_input: 0x1234_5678,
            special_return: 0xDEAD_BEEF,
            ..Default::default()
        };
        assert_eq!(bt_stage83_low20_fullword_gate(5, 0xF_FFFF, 0x3000, &mut b), 0xDEAD_BEEF);
        assert_eq!(b.events, [
            ("read_input", 0x3000, 0),
            ("read_input", 0x3000, 0),
            ("read_word", STAGE83_BT_SPECIAL_INPUT_ADDR, 0),
            ("special", 0x1234_5678, 0),
            ("write_word", STAGE83_BT_PUBLISH_WORD_ADDR, 1),
            ("write_byte", STAGE83_BT_READY_FLAG_ADDR, 1),
        ]);
    }

    #[test]
    fn full_r1_is_compared_against_only_low20_snapshot() {
        let mut b = B { input_values: std::vec![0x000F_FFFF], repair: 2, stage81_return: 11, ..Default::default() };
        assert_eq!(bt_stage83_low20_fullword_gate(0, 0x001F_FFFF, 7, &mut b), 11);
        assert_eq!(b.input_reads.len(), 1);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE83_CURRENT_BT_LOW20_GATE_ADDR, 0x171D08);
        assert_eq!(STAGE83_BT_SPECIAL_BOUNDARY, 0xBAA08);
        assert_eq!(STAGE83_BT_STAGE81_TAIL_ADDR, 0x171B84);
        assert_eq!(STAGE83_BT_EXACT_WORD, 0x200F_FFFF);
        assert_eq!(STAGE83_BT_SPECIAL_INPUT_ADDR, 0x352608);
        assert_eq!(STAGE83_BT_PUBLISH_WORD_ADDR, 0x222E00);
        assert_eq!(STAGE83_BT_READY_FLAG_ADDR, 0x2170EE);
        assert_eq!(STAGE83_BT_REJECT_FLAG_ADDR, 0x2170EF);
        assert_eq!(STAGE83_BT_REPAIR_INPUT_ADDR, 0x204B10);
    }
}

/// Stage 84: current fixed callback-slot publisher at `0x1720D8`.
///
/// The executable body is only eight bytes and uses a common literal-store form,
/// so provenance is frozen over the 16-byte body-plus-literal context. The
/// destination literal is stable at `0x2166D4`; the raw Thumb callback pointer
/// relocates from current `0x171FD9` to public-legacy structural `0x16DF29`.
pub const STAGE84_CURRENT_BT_CALLBACK_SLOT_PUBLISH_ADDR: u32 = 0x0017_20D8;
pub const STAGE84_BT_CALLBACK_SLOT_ADDR: u32 = 0x0021_66D4;
pub const STAGE84_BT_CALLBACK_THUMB: u32 = 0x0017_1FD9;

pub trait BtStage84Backend {
    fn write_word(&mut self, address: u32, value: u32);
}

/// Safe source-level model of current `0x1720D8`.
///
/// Firmware loads the fixed slot address and raw Thumb callback pointer, writes
/// the pointer once, and returns with R0 untouched.
pub fn bt_stage84_publish_callback<B: BtStage84Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    backend.write_word(STAGE84_BT_CALLBACK_SLOT_ADDR, STAGE84_BT_CALLBACK_THUMB);
    incoming_r0
}

#[cfg(test)]
mod stage84_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        writes: Vec<(u32, u32)>,
    }

    impl BtStage84Backend for B {
        fn write_word(&mut self, address: u32, value: u32) {
            self.writes.push((address, value));
        }
    }

    #[test]
    fn writes_exact_raw_thumb_pointer_once_and_preserves_r0() {
        let mut b = B::default();
        assert_eq!(bt_stage84_publish_callback(0xDEAD_BEEF, &mut b), 0xDEAD_BEEF);
        assert_eq!(b.writes, [(STAGE84_BT_CALLBACK_SLOT_ADDR, STAGE84_BT_CALLBACK_THUMB)]);
    }

    #[test]
    fn provenance_constants_freeze_current_slot_and_callback() {
        assert_eq!(STAGE84_CURRENT_BT_CALLBACK_SLOT_PUBLISH_ADDR, 0x1720D8);
        assert_eq!(STAGE84_BT_CALLBACK_SLOT_ADDR, 0x2166D4);
        assert_eq!(STAGE84_BT_CALLBACK_THUMB, 0x171FD9);
    }
}

/// Stage 85: current two-halfword circular-distance helper at `0x17192C`.
///
/// The exact 26-byte leaf plus its two literal dwords forms a unique 34-byte
/// current/public-legacy identity. The helper reads two fixed halfwords, uses
/// unsigned comparisons, and applies literal 100 only when the first value is
/// strictly greater than the second.
pub const STAGE85_CURRENT_BT_CIRCULAR_DISTANCE_ADDR: u32 = 0x0017_192C;
pub const STAGE85_BT_FIRST_HALFWORD_ADDR: u32 = 0x0021_70EC;
pub const STAGE85_BT_SECOND_HALFWORD_ADDR: u32 = 0x0021_7170;
pub const STAGE85_BT_WRAP_ADDEND: u32 = 100;

pub trait BtStage85Backend {
    fn read_halfword(&mut self, address: u32) -> u16;
}

/// Safe source-level model of current `0x17192C`.
///
/// Firmware reads the first halfword before the second. When first < second it
/// returns `second-first`; equality returns zero. When first > second it first
/// adds literal 100 to the zero-extended second value and then subtracts the
/// first using ordinary 32-bit arithmetic. No local range/modulo guard is
/// invented, so out-of-range values preserve the observable wrapping result.
pub fn bt_stage85_circular_distance<B: BtStage85Backend>(backend: &mut B) -> u32 {
    let first = u32::from(backend.read_halfword(STAGE85_BT_FIRST_HALFWORD_ADDR));
    let second = u32::from(backend.read_halfword(STAGE85_BT_SECOND_HALFWORD_ADDR));

    if first < second {
        second.wrapping_sub(first)
    } else if first == second {
        0
    } else {
        second
            .wrapping_add(STAGE85_BT_WRAP_ADDEND)
            .wrapping_sub(first)
    }
}

#[cfg(test)]
mod stage85_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    struct B {
        first: u16,
        second: u16,
        reads: Vec<u32>,
    }

    impl BtStage85Backend for B {
        fn read_halfword(&mut self, address: u32) -> u16 {
            self.reads.push(address);
            if address == STAGE85_BT_FIRST_HALFWORD_ADDR { self.first } else { self.second }
        }
    }

    #[test]
    fn reads_first_then_second_and_plain_difference_when_first_is_lower() {
        let mut b = B { first: 20, second: 70, reads: Vec::new() };
        assert_eq!(bt_stage85_circular_distance(&mut b), 50);
        assert_eq!(b.reads, [STAGE85_BT_FIRST_HALFWORD_ADDR, STAGE85_BT_SECOND_HALFWORD_ADDR]);
    }

    #[test]
    fn equality_returns_zero() {
        let mut b = B { first: 55, second: 55, reads: Vec::new() };
        assert_eq!(bt_stage85_circular_distance(&mut b), 0);
    }

    #[test]
    fn greater_first_uses_literal_hundred_before_subtract() {
        let mut b = B { first: 90, second: 10, reads: Vec::new() };
        assert_eq!(bt_stage85_circular_distance(&mut b), 20);
    }

    #[test]
    fn out_of_range_greater_case_preserves_u32_wrap() {
        let mut b = B { first: 500, second: 10, reads: Vec::new() };
        assert_eq!(
            bt_stage85_circular_distance(&mut b),
            10u32.wrapping_add(100).wrapping_sub(500)
        );
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE85_CURRENT_BT_CIRCULAR_DISTANCE_ADDR, 0x17192C);
        assert_eq!(STAGE85_BT_FIRST_HALFWORD_ADDR, 0x2170EC);
        assert_eq!(STAGE85_BT_SECOND_HALFWORD_ADDR, 0x217170);
        assert_eq!(STAGE85_BT_WRAP_ADDEND, 100);
    }
}

/// Stage 86: current guarded table-step helper at `0x1718E8`.
///
/// The exact current 56-byte body is a relocation-normalized structural
/// counterpart of public-legacy `0x16DA30`. Two critical-state calls relocate;
/// all local arithmetic, global addresses, and table indexing remain fixed.
pub const STAGE86_CURRENT_BT_GUARDED_TABLE_STEP_ADDR: u32 = 0x0017_18E8;
pub const STAGE86_BT_CRITICAL_BOUNDARY: u32 = ROM_CRITICAL_STATE_SWAP_LIKE_ADDR;
pub const STAGE86_BT_FIRST_HALFWORD_ADDR: u32 = 0x0021_70EC;
pub const STAGE86_BT_SECOND_HALFWORD_ADDR: u32 = 0x0021_7170;
pub const STAGE86_BT_TABLE_BASE_ADDR: u32 = 0x0022_2E34;
pub const STAGE86_BT_WRAP_LIMIT: u16 = 99;

pub trait BtStage86Backend {
    /// Current `0x780(value)`. The first call receives literal one and returns
    /// the token passed to the second call. The second return is ignored.
    fn critical_swap(&mut self, value: u32) -> u32;
    fn read_halfword(&mut self, address: u32) -> u16;
    fn write_halfword(&mut self, address: u32, value: u16);
    fn read_table_word(&mut self, address: u32) -> u32;
    fn write_output_word(&mut self, output_ptr: u32, value: u32);
}

/// Safe source-level model of current `0x1718E8`.
///
/// The two halfwords are read only after entering the critical-state boundary.
/// When they differ, firmware uses the original first halfword as an unchecked
/// table index, writes the selected dword through incoming R0, then increments
/// the first halfword with 16-bit wrap and resets values above 99 to zero.
/// The critical-state restore return is discarded; final R0 is the local
/// changed/not-changed boolean.
pub fn bt_stage86_guarded_table_step<B: BtStage86Backend>(
    output_ptr: u32,
    backend: &mut B,
) -> u32 {
    let token = backend.critical_swap(1);

    let first = backend.read_halfword(STAGE86_BT_FIRST_HALFWORD_ADDR);
    let second = backend.read_halfword(STAGE86_BT_SECOND_HALFWORD_ADDR);

    let changed = if second != first {
        let table_address = STAGE86_BT_TABLE_BASE_ADDR.wrapping_add(u32::from(first).wrapping_mul(4));
        let value = backend.read_table_word(table_address);
        backend.write_output_word(output_ptr, value);

        let incremented = first.wrapping_add(1);
        let next = if incremented > STAGE86_BT_WRAP_LIMIT { 0 } else { incremented };
        backend.write_halfword(STAGE86_BT_FIRST_HALFWORD_ADDR, next);
        1
    } else {
        0
    };

    let _ = backend.critical_swap(token);
    changed
}

#[cfg(test)]
mod stage86_tests {
    extern crate std;
    use super::*;
    use std::collections::BTreeMap;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        first: u16,
        second: u16,
        enter_token: u32,
        restore_return: u32,
        table: BTreeMap<u32, u32>,
        events: Vec<(&'static str, u32, u32)>,
    }

    impl BtStage86Backend for B {
        fn critical_swap(&mut self, value: u32) -> u32 {
            self.events.push(("critical", value, 0));
            if value == 1 { self.enter_token } else { self.restore_return }
        }
        fn read_halfword(&mut self, address: u32) -> u16 {
            self.events.push(("read_half", address, 0));
            if address == STAGE86_BT_FIRST_HALFWORD_ADDR { self.first } else { self.second }
        }
        fn write_halfword(&mut self, address: u32, value: u16) {
            self.events.push(("write_half", address, u32::from(value)));
            if address == STAGE86_BT_FIRST_HALFWORD_ADDR { self.first = value; }
        }
        fn read_table_word(&mut self, address: u32) -> u32 {
            self.events.push(("read_table", address, 0));
            self.table.get(&address).copied().unwrap_or_default()
        }
        fn write_output_word(&mut self, output_ptr: u32, value: u32) {
            self.events.push(("write_output", output_ptr, value));
        }
    }

    #[test]
    fn equal_halfwords_restore_critical_state_and_return_zero() {
        let mut b = B { first: 7, second: 7, enter_token: 0x55, restore_return: 0xFFFF_FFFF, ..Default::default() };
        assert_eq!(bt_stage86_guarded_table_step(0x9000, &mut b), 0);
        assert_eq!(b.events, [
            ("critical", 1, 0),
            ("read_half", STAGE86_BT_FIRST_HALFWORD_ADDR, 0),
            ("read_half", STAGE86_BT_SECOND_HALFWORD_ADDR, 0),
            ("critical", 0x55, 0),
        ]);
    }

    #[test]
    fn mismatch_indexes_with_original_first_then_increments_and_returns_one() {
        let first = 4u16;
        let address = STAGE86_BT_TABLE_BASE_ADDR + u32::from(first) * 4;
        let mut b = B { first, second: 9, enter_token: 0xA5, ..Default::default() };
        b.table.insert(address, 0x1234_5678);
        assert_eq!(bt_stage86_guarded_table_step(0xCAFE_0000, &mut b), 1);
        assert_eq!(b.first, 5);
        assert_eq!(b.events, [
            ("critical", 1, 0),
            ("read_half", STAGE86_BT_FIRST_HALFWORD_ADDR, 0),
            ("read_half", STAGE86_BT_SECOND_HALFWORD_ADDR, 0),
            ("read_table", address, 0),
            ("write_output", 0xCAFE_0000, 0x1234_5678),
            ("write_half", STAGE86_BT_FIRST_HALFWORD_ADDR, 5),
            ("critical", 0xA5, 0),
        ]);
    }

    #[test]
    fn increment_above_99_resets_to_zero() {
        let first = 99u16;
        let address = STAGE86_BT_TABLE_BASE_ADDR + u32::from(first) * 4;
        let mut b = B { first, second: 0, enter_token: 8, ..Default::default() };
        b.table.insert(address, 3);
        assert_eq!(bt_stage86_guarded_table_step(1, &mut b), 1);
        assert_eq!(b.first, 0);
    }

    #[test]
    fn u16_wrap_is_applied_before_the_greater_than_99_check() {
        let first = u16::MAX;
        let address = STAGE86_BT_TABLE_BASE_ADDR.wrapping_add(u32::from(first).wrapping_mul(4));
        let mut b = B { first, second: 0, enter_token: 2, ..Default::default() };
        b.table.insert(address, 7);
        assert_eq!(bt_stage86_guarded_table_step(2, &mut b), 1);
        assert_eq!(b.first, 0);
        assert!(b.events.contains(&("read_table", address, 0)));
    }

    #[test]
    fn restore_return_is_ignored_and_local_boolean_is_final() {
        let mut b = B { first: 1, second: 2, enter_token: 0x11, restore_return: 0xDEAD_BEEF, ..Default::default() };
        assert_eq!(bt_stage86_guarded_table_step(3, &mut b), 1);
        assert_eq!(b.events.last(), Some(&("critical", 0x11, 0)));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE86_CURRENT_BT_GUARDED_TABLE_STEP_ADDR, 0x1718E8);
        assert_eq!(STAGE86_BT_CRITICAL_BOUNDARY, 0x780);
        assert_eq!(STAGE86_BT_FIRST_HALFWORD_ADDR, 0x2170EC);
        assert_eq!(STAGE86_BT_SECOND_HALFWORD_ADDR, 0x217170);
        assert_eq!(STAGE86_BT_TABLE_BASE_ADDR, 0x222E34);
        assert_eq!(STAGE86_BT_WRAP_LIMIT, 99);
    }
}

/// Stage 87: current staged-init/control wrapper at `0x171C78`.
///
/// The exact 118-byte current body is a relocation-normalized structural
/// counterpart of public-legacy `0x16DBC4`. Stage 79, Stage 85, Stage 86,
/// and Stage 81 are already reconstructed dependencies; the remaining runtime
/// entries stay explicit opaque boundaries.
pub const STAGE87_CURRENT_BT_STAGED_CONTROL_ADDR: u32 = 0x0017_1C78;
pub const STAGE87_BT_CANARY_ADDR: u32 = 0x0020_0890;
pub const STAGE87_BT_INITIALIZED_ADDR: u32 = 0x0022_2E00;
pub const STAGE87_BT_THRESHOLD_HALFWORD_ADDR: u32 = 0x0020_4B14;
pub const STAGE87_BT_FALLBACK_GATE_ADDR: u32 = 0x0022_2DF8;
pub const STAGE87_BT_SHIFT_SOURCE_ADDR: u32 = 0x0020_4B10;
pub const STAGE87_BT_CHAIN_A_BOUNDARY: u32 = 0x000B_AA08;
pub const STAGE87_BT_CHAIN_B_BOUNDARY: u32 = 0x000B_A988;
pub const STAGE87_BT_ZERO_RESULT_BOUNDARY: u32 = 0x000B_DDBC;
pub const STAGE87_BT_CANARY_FAIL_BOUNDARY: u32 = 0x0000_94C0;
pub const STAGE87_BT_STAGE79_ADDR: u32 = STAGE79_CURRENT_BT_SNAPSHOT_FOLD_ADDR;
pub const STAGE87_BT_STAGE85_ADDR: u32 = STAGE85_CURRENT_BT_CIRCULAR_DISTANCE_ADDR;
pub const STAGE87_BT_STAGE86_ADDR: u32 = STAGE86_CURRENT_BT_GUARDED_TABLE_STEP_ADDR;
pub const STAGE87_BT_STAGE81_ADDR: u32 = STAGE81_CURRENT_BT_CRITICAL_REPAIR_ADDR;

pub trait BtStage87Backend {
    fn read_word(&mut self, address: u32) -> u32;
    fn write_word(&mut self, address: u32, value: u32);
    fn read_halfword(&mut self, address: u32) -> u16;
    fn read_byte(&mut self, address: u32) -> u8;

    fn stage79_snapshot_fold(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;
    fn chain_a(&mut self, value: u32) -> u32;
    fn chain_b(&mut self, value: u32) -> u32;

    /// Already-recovered Stage 85 fixed-global distance helper.
    fn stage85_distance(&mut self) -> u32;

    /// Already-recovered Stage 86. The stack-local dword is passed by pointer
    /// in firmware and may be replaced by the helper when it returns one.
    fn stage86_step(&mut self, local_word: &mut u32) -> u32;

    /// Current opaque `0xBDDBC`, reached only with current R0 equal to zero.
    /// Its return is overwritten by the following Stage-85 call.
    fn zero_result_boundary(&mut self, current_r0: u32) -> u32;

    /// Already-recovered Stage 81. Its return is ignored locally.
    fn stage81_repair(&mut self, value: u32) -> u32;

    /// Current hardening boundary `0x94C0`, invoked when the saved canary
    /// differs from the final reread. Firmware has the local result in R0.
    fn canary_fail(&mut self, local_word: u32);
}

/// Safe source-level model of current `0x171C78`.
///
/// `local_word` represents stack slot zero. It begins as incoming R0, may be
/// replaced by Stage 86 through pointer aliasing, or by the conditional
/// `0xBA988` call. The function returns this stack-local value. Threshold and
/// canary values are reread exactly where the firmware rereads them.
pub fn bt_stage87_staged_control<B: BtStage87Backend>(
    incoming_r0: u32,
    incoming_r1: u32,
    backend: &mut B,
) -> u32 {
    let canary_before = backend.read_word(STAGE87_BT_CANARY_ADDR);
    let mut local_word = incoming_r0;

    if backend.read_word(STAGE87_BT_INITIALIZED_ADDR) == 0 {
        let mut value = backend.stage79_snapshot_fold(
            incoming_r0,
            incoming_r1,
            0,
            STAGE87_BT_CANARY_ADDR,
        );
        value = backend.chain_a(value);
        value = backend.chain_b(value);
        value = backend.chain_b(value);
        let _ = backend.chain_b(value);
        backend.write_word(STAGE87_BT_INITIALIZED_ADDR, 1);
    }

    let mut current_r0 = incoming_r0;
    let mut use_stage86 = incoming_r0 != 0;

    if !use_stage86 {
        current_r0 = backend.stage85_distance();
        let threshold = u32::from(backend.read_halfword(STAGE87_BT_THRESHOLD_HALFWORD_ADDR));
        if current_r0 > threshold {
            use_stage86 = true;
        }
    }

    if use_stage86 {
        current_r0 = backend.stage86_step(&mut local_word);
        if current_r0 == 0 {
            if backend.read_byte(STAGE87_BT_FALLBACK_GATE_ADDR) != 0 {
                let _ = backend.zero_result_boundary(0);
            } else {
                local_word = backend.chain_b(0);
            }
        }
    } else {
        // This is the branch from the first Stage-85 comparison when
        // distance <= the live threshold.
        local_word = backend.chain_b(current_r0);
    }

    let distance_after = backend.stage85_distance();
    let threshold_after = u32::from(backend.read_halfword(STAGE87_BT_THRESHOLD_HALFWORD_ADDR));
    if distance_after <= threshold_after {
        let value = backend.read_word(STAGE87_BT_SHIFT_SOURCE_ADDR) >> 1;
        let _ = backend.stage81_repair(value);
    }

    let canary_after = backend.read_word(STAGE87_BT_CANARY_ADDR);
    if canary_before != canary_after {
        backend.canary_fail(local_word);
    }

    local_word
}

#[cfg(test)]
mod stage87_tests {
    extern crate std;
    use super::*;
    use std::vec;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        canary_reads: Vec<u32>,
        initialized: u32,
        thresholds: Vec<u16>,
        fallback_gate: u8,
        shift_source: u32,
        stage79_return: u32,
        chain_returns: Vec<u32>,
        stage85_returns: Vec<u32>,
        stage86_return: u32,
        stage86_write: Option<u32>,
        events: Vec<(&'static str, u32, u32, u32)>,
    }

    impl B {
        fn pop_word(v: &mut Vec<u32>) -> u32 { if v.is_empty() { 0 } else { v.remove(0) } }
        fn pop_half(v: &mut Vec<u16>) -> u16 { if v.is_empty() { 0 } else { v.remove(0) } }
    }

    impl BtStage87Backend for B {
        fn read_word(&mut self, address: u32) -> u32 {
            self.events.push(("read_word", address, 0, 0));
            match address {
                STAGE87_BT_CANARY_ADDR => Self::pop_word(&mut self.canary_reads),
                STAGE87_BT_INITIALIZED_ADDR => self.initialized,
                STAGE87_BT_SHIFT_SOURCE_ADDR => self.shift_source,
                _ => 0,
            }
        }
        fn write_word(&mut self, address: u32, value: u32) {
            self.events.push(("write_word", address, value, 0));
            if address == STAGE87_BT_INITIALIZED_ADDR { self.initialized = value; }
        }
        fn read_halfword(&mut self, address: u32) -> u16 {
            self.events.push(("read_half", address, 0, 0));
            Self::pop_half(&mut self.thresholds)
        }
        fn read_byte(&mut self, address: u32) -> u8 {
            self.events.push(("read_byte", address, 0, 0));
            self.fallback_gate
        }
        fn stage79_snapshot_fold(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->u32 {
            self.events.push(("stage79", r0, r1, r2));
            self.events.push(("stage79_r3", r3, 0, 0));
            self.stage79_return
        }
        fn chain_a(&mut self, value:u32)->u32 {
            self.events.push(("chain_a", value, 0, 0));
            Self::pop_word(&mut self.chain_returns)
        }
        fn chain_b(&mut self, value:u32)->u32 {
            self.events.push(("chain_b", value, 0, 0));
            Self::pop_word(&mut self.chain_returns)
        }
        fn stage85_distance(&mut self)->u32 {
            self.events.push(("stage85", 0, 0, 0));
            Self::pop_word(&mut self.stage85_returns)
        }
        fn stage86_step(&mut self, local_word:&mut u32)->u32 {
            self.events.push(("stage86", *local_word, 0, 0));
            if let Some(v)=self.stage86_write { *local_word=v; }
            self.stage86_return
        }
        fn zero_result_boundary(&mut self,current_r0:u32)->u32 {
            self.events.push(("zero_boundary", current_r0, 0, 0));
            0xFFFF_FFFF
        }
        fn stage81_repair(&mut self,value:u32)->u32 {
            self.events.push(("stage81", value, 0, 0));
            0xDEAD_BEEF
        }
        fn canary_fail(&mut self,local_word:u32) {
            self.events.push(("canary_fail", local_word, 0, 0));
        }
    }

    #[test]
    fn one_time_init_threads_stage79_through_a_and_three_b_calls_then_sets_flag() {
        let mut b=B {
            canary_reads:vec![7,7],
            initialized:0,
            stage79_return:10,
            chain_returns:vec![11,12,13,14,99],
            stage85_returns:vec![1,200],
            thresholds:vec![5,5],
            ..Default::default()
        };
        // incoming_r0 zero: first distance 1 <=5, so the fifth chain-b return
        // becomes local_word; second distance 200 skips Stage81.
        assert_eq!(bt_stage87_staged_control(0,2,&mut b),99);
        assert_eq!(b.initialized,1);
        let init_pos=b.events.iter().position(|e|e.0=="write_word"&&e.1==STAGE87_BT_INITIALIZED_ADDR).unwrap();
        let names:Vec<_>=b.events[..init_pos].iter().filter(|e|e.0=="stage79"||e.0=="chain_a"||e.0=="chain_b").map(|e|e.0).collect();
        assert_eq!(names,["stage79","chain_a","chain_b","chain_b","chain_b"]);
    }

    #[test]
    fn nonzero_input_goes_directly_to_stage86_and_changed_write_becomes_final_local() {
        let mut b=B {
            canary_reads:vec![1,1],
            initialized:1,
            stage86_return:1,
            stage86_write:Some(0xCAFE_BABE),
            stage85_returns:vec![999],
            thresholds:vec![3],
            ..Default::default()
        };
        assert_eq!(bt_stage87_staged_control(0x55,0,&mut b),0xCAFE_BABE);
        assert!(!b.events.iter().any(|e|e.0=="read_byte"||e.0=="zero_boundary"));
    }

    #[test]
    fn zero_stage86_with_zero_gate_replaces_local_with_chain_return() {
        let mut b=B {
            canary_reads:vec![2,2],
            initialized:1,
            stage85_returns:vec![10],
            thresholds:vec![2],
            stage86_return:0,
            fallback_gate:0,
            chain_returns:vec![0x1234],
            ..Default::default()
        };
        assert_eq!(bt_stage87_staged_control(0,0,&mut b),0x1234);
        assert!(b.events.iter().any(|e|*e==("chain_b",0,0,0)));
    }

    #[test]
    fn zero_stage86_with_nonzero_gate_calls_opaque_boundary_and_keeps_local() {
        let mut b=B {
            canary_reads:vec![3,3],
            initialized:1,
            stage85_returns:vec![10],
            thresholds:vec![2],
            stage86_return:0,
            fallback_gate:1,
            ..Default::default()
        };
        assert_eq!(bt_stage87_staged_control(0x44,0,&mut b),0x44);
        assert!(b.events.iter().any(|e|*e==("zero_boundary",0,0,0)));
    }

    #[test]
    fn second_distance_rereads_threshold_and_conditionally_calls_stage81_with_shifted_word() {
        let mut b=B {
            canary_reads:vec![4,4],
            initialized:1,
            stage85_returns:vec![3,4],
            thresholds:vec![2,5],
            stage86_return:1,
            stage86_write:Some(9),
            shift_source:0x8000_0003,
            ..Default::default()
        };
        assert_eq!(bt_stage87_staged_control(0,0,&mut b),9);
        assert!(b.events.iter().any(|e|*e==("stage81",0x4000_0001,0,0)));
    }

    #[test]
    fn canary_is_reread_after_all_work_and_mismatch_receives_final_local() {
        let mut b=B {
            canary_reads:vec![0x11,0x22],
            initialized:1,
            stage85_returns:vec![0,99],
            thresholds:vec![1,1],
            chain_returns:vec![0xABCD],
            ..Default::default()
        };
        assert_eq!(bt_stage87_staged_control(0,0,&mut b),0xABCD);
        assert_eq!(b.events.last(),Some(&("canary_fail",0xABCD,0,0)));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE87_CURRENT_BT_STAGED_CONTROL_ADDR,0x171C78);
        assert_eq!(STAGE87_BT_CANARY_ADDR,0x200890);
        assert_eq!(STAGE87_BT_INITIALIZED_ADDR,0x222E00);
        assert_eq!(STAGE87_BT_THRESHOLD_HALFWORD_ADDR,0x204B14);
        assert_eq!(STAGE87_BT_FALLBACK_GATE_ADDR,0x222DF8);
        assert_eq!(STAGE87_BT_SHIFT_SOURCE_ADDR,0x204B10);
        assert_eq!(STAGE87_BT_STAGE79_ADDR,0x1719E0);
        assert_eq!(STAGE87_BT_STAGE85_ADDR,0x17192C);
        assert_eq!(STAGE87_BT_STAGE86_ADDR,0x1718E8);
        assert_eq!(STAGE87_BT_STAGE81_ADDR,0x171B84);
    }
}

/// Stage 88: current wait/reset wrapper at `0x171D68`.
///
/// The exact current body is 92 bytes. No public-legacy structural counterpart is
/// promoted: current semantics are derived only from the canonical 91900-byte HCD.
/// The model preserves the initial low-20 gate, mandatory full-word rereads, the
/// unbounded local wait loop, path-dependent R0 forwarding into `0xBAAE4`, the
/// critical-state token restore, and the mismatch tail into already recovered Stage 81.
pub const STAGE88_CURRENT_BT_WAIT_RESET_ADDR: u32 = 0x0017_1D68;
pub const STAGE88_BT_STATUS_WORD_ADDR: u32 = 0x0035_2604;
pub const STAGE88_BT_LOW20_EXPECTED: u32 = 0x000F_FFFF;
pub const STAGE88_BT_FULL_EXPECTED: u32 = 0x200F_FFFF;
pub const STAGE88_BT_STATE_WORD_ADDR: u32 = 0x0035_2600;
pub const STAGE88_BT_STATE_WORD_VALUE: u32 = 3;
pub const STAGE88_BT_SECOND_RESET_WORD_ADDR: u32 = 0x0035_2614;
pub const STAGE88_BT_REJECT_FLAG_ADDR: u32 = 0x0021_70EF;
pub const STAGE88_BT_READY_FLAG_ADDR: u32 = 0x0021_70EE;
pub const STAGE88_BT_REPAIR_INPUT_ADDR: u32 = 0x0020_4B10;
pub const STAGE88_BT_WAIT_BOUNDARY: u32 = 0x000B_0210;
pub const STAGE88_BT_PREP_BOUNDARY: u32 = 0x000B_AAE4;
pub const STAGE88_BT_CRITICAL_BOUNDARY: u32 = 0x0000_0780;
pub const STAGE88_BT_STAGE81_TAIL: u32 = STAGE81_CURRENT_BT_CRITICAL_REPAIR_ADDR;

pub trait BtStage88Backend {
    fn read_word(&mut self, address: u32) -> u32;
    fn write_word(&mut self, address: u32, value: u32);
    fn write_byte(&mut self, address: u32, value: u8);

    /// Opaque current `0xB0210`. The body reaches it with the live caller
    /// registers shown here; its return becomes the next live R0.
    fn wait_boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;

    /// Opaque current `0xBAAE4`. Its return is immediately overwritten.
    fn prep_boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;

    /// Current critical-state boundary `0x780(value)`.
    fn critical_swap(&mut self, value: u32) -> u32;

    /// Already recovered Stage-81 tail at current `0x171B84`.
    fn stage81_tail(&mut self, value: u32) -> u32;
}

/// Safe source-level model of current `0x171D68`.
///
/// The first status read is used only for the low-20 gate. On the match path the
/// full status is reread before every exact-word test, so the model deliberately
/// does not reuse the first snapshot. There is no local retry bound.
pub fn bt_stage88_wait_reset<B: BtStage88Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    let first = backend.read_word(STAGE88_BT_STATUS_WORD_ADDR);
    let initial_low20 = first & STAGE88_BT_LOW20_EXPECTED;

    if initial_low20 != STAGE88_BT_LOW20_EXPECTED {
        backend.write_byte(STAGE88_BT_REJECT_FLAG_ADDR, 0);
        let repair = backend.read_word(STAGE88_BT_REPAIR_INPUT_ADDR);
        return backend.stage81_tail(repair >> 1);
    }

    let mut live_r0 = incoming_r0;
    loop {
        let full = backend.read_word(STAGE88_BT_STATUS_WORD_ADDR);
        if full == STAGE88_BT_FULL_EXPECTED {
            break;
        }

        live_r0 = backend.wait_boundary(
            live_r0,
            STAGE88_BT_LOW20_EXPECTED,
            initial_low20,
            full,
        );
    }

    backend.write_word(STAGE88_BT_STATE_WORD_ADDR, STAGE88_BT_STATE_WORD_VALUE);

    let _ = backend.prep_boundary(
        live_r0,
        STAGE88_BT_LOW20_EXPECTED,
        STAGE88_BT_STATE_WORD_VALUE,
        STAGE88_BT_STATE_WORD_ADDR,
    );

    let token = backend.critical_swap(1);

    backend.write_word(STAGE88_BT_STATUS_WORD_ADDR, 0);
    backend.write_word(STAGE88_BT_SECOND_RESET_WORD_ADDR, 0);
    backend.write_byte(STAGE88_BT_REJECT_FLAG_ADDR, 0);
    backend.write_byte(STAGE88_BT_READY_FLAG_ADDR, 0);

    backend.critical_swap(token)
}

#[cfg(test)]
mod stage88_tests {
    extern crate std;
    use super::*;
    use std::vec;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        status_reads: Vec<u32>,
        repair: u32,
        wait_returns: Vec<u32>,
        wait_index: usize,
        prep_return: u32,
        critical_returns: Vec<u32>,
        critical_index: usize,
        stage81_return: u32,
        events: Vec<(&'static str, u32, u32, u32, u32)>,
    }

    impl BtStage88Backend for B {
        fn read_word(&mut self, address: u32) -> u32 {
            if address == STAGE88_BT_STATUS_WORD_ADDR {
                let value = self.status_reads.remove(0);
                self.events.push(("read_status", value, 0, 0, 0));
                value
            } else {
                assert_eq!(address, STAGE88_BT_REPAIR_INPUT_ADDR);
                self.events.push(("read_repair", self.repair, 0, 0, 0));
                self.repair
            }
        }

        fn write_word(&mut self, address: u32, value: u32) {
            self.events.push(("write_word", address, value, 0, 0));
        }

        fn write_byte(&mut self, address: u32, value: u8) {
            self.events.push(("write_byte", address, value as u32, 0, 0));
        }

        fn wait_boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32 {
            self.events.push(("wait", r0, r1, r2, r3));
            let value = self.wait_returns[self.wait_index];
            self.wait_index += 1;
            value
        }

        fn prep_boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32 {
            self.events.push(("prep", r0, r1, r2, r3));
            self.prep_return
        }

        fn critical_swap(&mut self, value: u32) -> u32 {
            self.events.push(("critical", value, 0, 0, 0));
            let ret = self.critical_returns[self.critical_index];
            self.critical_index += 1;
            ret
        }

        fn stage81_tail(&mut self, value: u32) -> u32 {
            self.events.push(("stage81", value, 0, 0, 0));
            self.stage81_return
        }
    }

    #[test]
    fn low20_mismatch_clears_reject_and_tails_stage81() {
        let mut b = B {
            status_reads: vec![0x1234_5678],
            repair: 0x20,
            stage81_return: 0xABCD,
            ..Default::default()
        };

        assert_eq!(bt_stage88_wait_reset(0xDEAD_BEEF, &mut b), 0xABCD);
        assert_eq!(b.events, [
            ("read_status", 0x1234_5678, 0, 0, 0),
            ("write_byte", STAGE88_BT_REJECT_FLAG_ADDR, 0, 0, 0),
            ("read_repair", 0x20, 0, 0, 0),
            ("stage81", 0x10, 0, 0, 0),
        ]);
    }

    #[test]
    fn exact_reread_without_wait_forwards_incoming_r0_into_prep() {
        let mut b = B {
            status_reads: vec![0x000F_FFFF, STAGE88_BT_FULL_EXPECTED],
            prep_return: 0xDEAD_BEEF,
            critical_returns: vec![0x77, 0x88],
            ..Default::default()
        };

        assert_eq!(bt_stage88_wait_reset(0x1234_5678, &mut b), 0x88);
        assert!(b.events.contains(&(
            "prep",
            0x1234_5678,
            STAGE88_BT_LOW20_EXPECTED,
            3,
            STAGE88_BT_STATE_WORD_ADDR,
        )));
        assert!(b.events.contains(&("critical", 1, 0, 0, 0)));
        assert!(b.events.contains(&("critical", 0x77, 0, 0, 0)));
        assert!(!b.events.iter().any(|e| e.0 == "wait"));
    }

    #[test]
    fn final_wait_return_becomes_prep_r0_and_wait_reuses_initial_low20() {
        let mut b = B {
            status_reads: vec![
                0x000F_FFFF,
                0x100F_FFFF,
                0x300F_FFFF,
                STAGE88_BT_FULL_EXPECTED,
            ],
            wait_returns: vec![0x1111, 0x2222],
            critical_returns: vec![0x3333, 0x4444],
            ..Default::default()
        };

        assert_eq!(bt_stage88_wait_reset(0xAAAA, &mut b), 0x4444);
        assert!(b.events.contains(&(
            "wait",
            0xAAAA,
            STAGE88_BT_LOW20_EXPECTED,
            STAGE88_BT_LOW20_EXPECTED,
            0x100F_FFFF,
        )));
        assert!(b.events.contains(&(
            "wait",
            0x1111,
            STAGE88_BT_LOW20_EXPECTED,
            STAGE88_BT_LOW20_EXPECTED,
            0x300F_FFFF,
        )));
        assert!(b.events.contains(&(
            "prep",
            0x2222,
            STAGE88_BT_LOW20_EXPECTED,
            3,
            STAGE88_BT_STATE_WORD_ADDR,
        )));
    }

    #[test]
    fn reset_order_preserves_critical_token_and_ignores_prep_return() {
        let mut b = B {
            status_reads: vec![STAGE88_BT_FULL_EXPECTED, STAGE88_BT_FULL_EXPECTED],
            prep_return: 0xFFFF_FFFF,
            critical_returns: vec![0xCAFE_BABE, 0x1357_2468],
            ..Default::default()
        };

        assert_eq!(bt_stage88_wait_reset(9, &mut b), 0x1357_2468);

        let expected_tail = [
            ("write_word", STAGE88_BT_STATE_WORD_ADDR, 3, 0, 0),
            ("prep", 9, STAGE88_BT_LOW20_EXPECTED, 3, STAGE88_BT_STATE_WORD_ADDR),
            ("critical", 1, 0, 0, 0),
            ("write_word", STAGE88_BT_STATUS_WORD_ADDR, 0, 0, 0),
            ("write_word", STAGE88_BT_SECOND_RESET_WORD_ADDR, 0, 0, 0),
            ("write_byte", STAGE88_BT_REJECT_FLAG_ADDR, 0, 0, 0),
            ("write_byte", STAGE88_BT_READY_FLAG_ADDR, 0, 0, 0),
            ("critical", 0xCAFE_BABE, 0, 0, 0),
        ];
        assert_eq!(&b.events[b.events.len() - expected_tail.len()..], &expected_tail);
    }

    #[test]
    fn provenance_constants_are_current_only() {
        assert_eq!(STAGE88_CURRENT_BT_WAIT_RESET_ADDR, 0x171D68);
        assert_eq!(STAGE88_BT_STATUS_WORD_ADDR, 0x352604);
        assert_eq!(STAGE88_BT_LOW20_EXPECTED, 0xFFFFF);
        assert_eq!(STAGE88_BT_FULL_EXPECTED, 0x200FFFFF);
        assert_eq!(STAGE88_BT_STATE_WORD_ADDR, 0x352600);
        assert_eq!(STAGE88_BT_REJECT_FLAG_ADDR, 0x2170EF);
        assert_eq!(STAGE88_BT_READY_FLAG_ADDR, 0x2170EE);
        assert_eq!(STAGE88_BT_REPAIR_INPUT_ADDR, 0x204B10);
        assert_eq!(STAGE88_BT_WAIT_BOUNDARY, 0xB0210);
        assert_eq!(STAGE88_BT_PREP_BOUNDARY, 0xBAAE4);
        assert_eq!(STAGE88_BT_CRITICAL_BOUNDARY, 0x780);
        assert_eq!(STAGE88_BT_STAGE81_TAIL, 0x171B84);
    }
}

/// Stage 89: current fixed-global tail thunk at `0x171DE0`.
///
/// The exact current function body is eight bytes. The adjacent literal at
/// `0x171DE8` resolves to `0x222078`; it is data, not part of the body.
/// Firmware loads R3 with that address, loads R0 from `*R3`, and tail-branches
/// to opaque current boundary `0x13218`. Incoming R1/R2 survive; R3 does not.
pub const STAGE89_CURRENT_BT_FIXED_GLOBAL_TAIL_ADDR: u32 = 0x0017_1DE0;
pub const STAGE89_BT_GLOBAL_WORD_ADDR: u32 = 0x0022_2078;
pub const STAGE89_BT_TAIL_BOUNDARY: u32 = 0x0001_3218;

pub trait BtStage89Backend {
    fn read_global_word(&mut self, address: u32) -> u32;
    fn tail_boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;
}

/// Safe source-level model of current `0x171DE0`.
///
/// The incoming R0 and R3 values are discarded by the thunk. R1/R2 are
/// forwarded unchanged. The tail boundary sees R3 equal to the literal address
/// used by the load, and its return is the thunk's final return.
pub fn bt_stage89_fixed_global_tail<B: BtStage89Backend>(
    _incoming_r0: u32,
    incoming_r1: u32,
    incoming_r2: u32,
    _incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let value = backend.read_global_word(STAGE89_BT_GLOBAL_WORD_ADDR);
    backend.tail_boundary(
        value,
        incoming_r1,
        incoming_r2,
        STAGE89_BT_GLOBAL_WORD_ADDR,
    )
}

#[cfg(test)]
mod stage89_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        word: u32,
        ret: u32,
        events: Vec<(&'static str, u32, u32, u32, u32)>,
    }

    impl BtStage89Backend for B {
        fn read_global_word(&mut self, address: u32) -> u32 {
            self.events.push(("read", address, 0, 0, 0));
            self.word
        }
        fn tail_boundary(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32 {
            self.events.push(("tail", r0, r1, r2, r3));
            self.ret
        }
    }

    #[test]
    fn overwrites_r0_and_r3_but_preserves_r1_r2_into_tail() {
        let mut b = B { word: 0xAABB_CCDD, ret: 0x1234_5678, ..Default::default() };
        assert_eq!(
            bt_stage89_fixed_global_tail(0xDEAD_BEEF, 0x11, 0x22, 0x33, &mut b),
            0x1234_5678
        );
        assert_eq!(b.events, [
            ("read", STAGE89_BT_GLOBAL_WORD_ADDR, 0, 0, 0),
            ("tail", 0xAABB_CCDD, 0x11, 0x22, STAGE89_BT_GLOBAL_WORD_ADDR),
        ]);
    }

    #[test]
    fn tail_return_is_final_without_local_transform() {
        let mut b = B { word: 0, ret: 0xFFFF_FF00, ..Default::default() };
        assert_eq!(bt_stage89_fixed_global_tail(1, 2, 3, 4, &mut b), 0xFFFF_FF00);
    }

    #[test]
    fn provenance_constants_are_current_only() {
        assert_eq!(STAGE89_CURRENT_BT_FIXED_GLOBAL_TAIL_ADDR, 0x171DE0);
        assert_eq!(STAGE89_BT_GLOBAL_WORD_ADDR, 0x222078);
        assert_eq!(STAGE89_BT_TAIL_BOUNDARY, 0x13218);
    }
}

/// Stage 90: bounded signed status poll at `0x171DEC`.
///
/// Firmware loads a fixed dword address from the adjacent literal, initializes
/// R0 to 100, and repeatedly reloads the same dword. A nonnegative signed value
/// returns literal one immediately. Negative values decrement the 8-bit-sized
/// loop counter in R0; after exactly 100 negative observations the return is zero.
pub const STAGE90_CURRENT_BT_BOUNDED_STATUS_POLL_ADDR: u32 = 0x0017_1DEC;
pub const STAGE90_BT_STATUS_WORD_ADDR: u32 = 0x0065_0318;
pub const STAGE90_BT_MAX_POLLS: u32 = 100;

pub trait BtStage90Backend {
    fn read_status_word(&mut self, address: u32) -> u32;
}

/// Safe source-level model of current `0x171DEC`.
///
/// The comparison is signed (`BGE` after `CMP R3,#0`). The same address is
/// reread every iteration; no snapshot or delay is invented.
pub fn bt_stage90_bounded_status_poll<B: BtStage90Backend>(backend: &mut B) -> u32 {
    let mut remaining = STAGE90_BT_MAX_POLLS;
    loop {
        let value = backend.read_status_word(STAGE90_BT_STATUS_WORD_ADDR);
        if (value as i32) >= 0 {
            return 1;
        }
        remaining -= 1;
        if remaining == 0 {
            return 0;
        }
    }
}

#[cfg(test)]
mod stage90_tests {
    extern crate std;
    use super::*;
    use std::vec;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        values: Vec<u32>,
        reads: usize,
    }

    impl BtStage90Backend for B {
        fn read_status_word(&mut self, address: u32) -> u32 {
            assert_eq!(address, STAGE90_BT_STATUS_WORD_ADDR);
            let value = self.values.get(self.reads).copied().unwrap_or(0xFFFF_FFFF);
            self.reads += 1;
            value
        }
    }

    #[test]
    fn first_nonnegative_returns_one_after_one_read() {
        let mut b = B { values: vec![0], ..Default::default() };
        assert_eq!(bt_stage90_bounded_status_poll(&mut b), 1);
        assert_eq!(b.reads, 1);
    }

    #[test]
    fn signed_negative_values_keep_polling_until_nonnegative() {
        let mut b = B { values: vec![0x8000_0000, 0xFFFF_FFFF, 7], ..Default::default() };
        assert_eq!(bt_stage90_bounded_status_poll(&mut b), 1);
        assert_eq!(b.reads, 3);
    }

    #[test]
    fn one_hundred_negative_reads_return_zero() {
        let mut b = B { values: vec![0xFFFF_FFFF; 100], ..Default::default() };
        assert_eq!(bt_stage90_bounded_status_poll(&mut b), 0);
        assert_eq!(b.reads, 100);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE90_CURRENT_BT_BOUNDED_STATUS_POLL_ADDR, 0x171DEC);
        assert_eq!(STAGE90_BT_STATUS_WORD_ADDR, 0x650318);
        assert_eq!(STAGE90_BT_MAX_POLLS, 100);
    }
}

/// Stage 91: bounded bit-30 poll at `0x171E04`.
///
/// Firmware loads the fixed dword at `0x650310`, shifts the live value left by
/// one, and branches on the resulting N flag. That tests original bit 30.
/// The read is repeated at most 100 times.
pub const STAGE91_CURRENT_BT_BOUNDED_BIT30_POLL_ADDR: u32 = 0x0017_1E04;
pub const STAGE91_BT_STATUS_WORD_ADDR: u32 = 0x0065_0310;
pub const STAGE91_BT_MAX_POLLS: u32 = 100;
pub const STAGE91_BT_TEST_BIT: u32 = 1 << 30;

pub trait BtStage91Backend {
    fn read_status_word(&mut self, address: u32) -> u32;
}

/// Safe source-level model of current `0x171E04`.
///
/// A set original bit 30 returns literal one immediately. Otherwise firmware
/// reloads the same dword until the 100-read budget is exhausted, then returns
/// zero. No delay or cached snapshot is introduced.
pub fn bt_stage91_bounded_bit30_poll<B: BtStage91Backend>(backend: &mut B) -> u32 {
    let mut remaining = STAGE91_BT_MAX_POLLS;
    loop {
        let value = backend.read_status_word(STAGE91_BT_STATUS_WORD_ADDR);
        if (value & STAGE91_BT_TEST_BIT) != 0 {
            return 1;
        }
        remaining -= 1;
        if remaining == 0 {
            return 0;
        }
    }
}

#[cfg(test)]
mod stage91_tests {
    extern crate std;
    use super::*;
    use std::vec;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        values: Vec<u32>,
        reads: usize,
    }

    impl BtStage91Backend for B {
        fn read_status_word(&mut self, address: u32) -> u32 {
            assert_eq!(address, STAGE91_BT_STATUS_WORD_ADDR);
            let v = self.values.get(self.reads).copied().unwrap_or(0);
            self.reads += 1;
            v
        }
    }

    #[test]
    fn bit30_set_returns_one_immediately() {
        let mut b = B { values: vec![1 << 30], ..Default::default() };
        assert_eq!(bt_stage91_bounded_bit30_poll(&mut b), 1);
        assert_eq!(b.reads, 1);
    }

    #[test]
    fn bit31_does_not_satisfy_the_shifted_sign_test() {
        let mut b = B { values: vec![1 << 31, 1 << 30], ..Default::default() };
        assert_eq!(bt_stage91_bounded_bit30_poll(&mut b), 1);
        assert_eq!(b.reads, 2);
    }

    #[test]
    fn one_hundred_clear_reads_return_zero() {
        let mut b = B { values: vec![0; 100], ..Default::default() };
        assert_eq!(bt_stage91_bounded_bit30_poll(&mut b), 0);
        assert_eq!(b.reads, 100);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE91_CURRENT_BT_BOUNDED_BIT30_POLL_ADDR, 0x171E04);
        assert_eq!(STAGE91_BT_STATUS_WORD_ADDR, 0x650310);
        assert_eq!(STAGE91_BT_TEST_BIT, 1 << 30);
        assert_eq!(STAGE91_BT_MAX_POLLS, 100);
    }
}

/// Stage 92: guarded four-byte transfer sequence at `0x171E1C`.
///
/// This wrapper composes the already reconstructed Stage-90 and Stage-91 poll
/// leaves. It also preserves the stack-canary check visible in the binary.
pub const STAGE92_CURRENT_BT_GUARDED_TRANSFER_ADDR: u32 = 0x0017_1E1C;
pub const STAGE92_BT_GUARD_ADDR: u32 = 0x0020_0890;
pub const STAGE92_BT_SNAPSHOT_ADDR: u32 = 0x0022_2554;
pub const STAGE92_BT_MODE_WORD_ADDR: u32 = 0x0065_0310;
pub const STAGE92_BT_BYTE_PUBLISH_ADDR: u32 = 0x0065_0328;
pub const STAGE92_BT_STAGE90_WORD_ADDR: u32 = 0x0065_0318;
pub const STAGE92_BT_CONTROL_WORD_ADDR: u32 = 0x0065_0314;
pub const STAGE92_BT_GUARD_FAIL_BOUNDARY: u32 = 0x0000_94C0;
pub const STAGE92_BT_STAGE90_WRITE_VALUE: u32 = 0x8100_0000;

pub trait BtStage92Backend {
    fn read_word(&mut self, address: u32) -> u32;
    fn write_word(&mut self, address: u32, value: u32);
    fn guard_fail_boundary(
        &mut self,
        current_r0: u32,
        current_r1: u32,
        saved_guard: u32,
        live_guard: u32,
    ) -> u32;
}

/// Safe source-level model of current `0x171E1C`.
///
/// The initial bit-4 gate is read from `0x650310`. On the active path the
/// snapshot at `0x222554` is consumed little-endian, one byte per iteration.
/// Each byte is widened to a dword at `0x650328`, `0x81000000` is written to
/// `0x650318`, then Stage 90 is invoked and its return ignored. Stage 91 is
/// invoked after all four bytes; its return is also ignored. Bit 3 is finally
/// ORed into `0x650314`.
///
/// The stack guard is checked on both result paths. If it changed, opaque
/// boundary `0x94C0` is called; if that boundary were to return, its R0 becomes
/// the wrapper's final return, matching the machine code.
pub fn bt_stage92_guarded_transfer<B>(
    backend: &mut B,
) -> u32
where
    B: BtStage92Backend + BtStage90Backend + BtStage91Backend,
{
    let saved_guard = backend.read_word(STAGE92_BT_GUARD_ADDR);
    let snapshot = backend.read_word(STAGE92_BT_SNAPSHOT_ADDR);
    let mode_word = <B as BtStage91Backend>::read_status_word(
        backend,
        STAGE92_BT_MODE_WORD_ADDR,
    );
    let active = (mode_word & 0x10) == 0;

    let (mut result, guard_r1) = if active {
        for byte in snapshot.to_le_bytes() {
            backend.write_word(STAGE92_BT_BYTE_PUBLISH_ADDR, u32::from(byte));
            backend.write_word(
                STAGE92_BT_STAGE90_WORD_ADDR,
                STAGE92_BT_STAGE90_WRITE_VALUE,
            );
            let _ = bt_stage90_bounded_status_poll(backend);
        }

        let _ = bt_stage91_bounded_bit30_poll(backend);

        let control = backend.read_word(STAGE92_BT_CONTROL_WORD_ADDR);
        backend.write_word(STAGE92_BT_CONTROL_WORD_ADDR, control | 0x8);
        (1u32, 4u32)
    } else {
        (0u32, 0x10u32)
    };

    let live_guard = backend.read_word(STAGE92_BT_GUARD_ADDR);
    if live_guard != saved_guard {
        result = backend.guard_fail_boundary(result, guard_r1, saved_guard, live_guard);
    }

    result
}

#[cfg(test)]
mod stage92_tests {
    extern crate std;
    use super::*;
    use std::collections::VecDeque;
    use std::vec;
    use std::vec::Vec;

    struct B {
        guard: u32,
        guard_reads: VecDeque<u32>,
        snapshot: u32,
        mode_reads: VecDeque<u32>,
        stage90_reads: VecDeque<u32>,
        control: u32,
        guard_fail_ret: u32,
        events: Vec<(&'static str, u32, u32)>,
    }

    impl Default for B {
        fn default() -> Self {
            Self {
                guard: 0x1122_3344,
                guard_reads: VecDeque::new(),
                snapshot: 0x4433_2211,
                mode_reads: VecDeque::from(vec![0, 1 << 30]),
                stage90_reads: VecDeque::from(vec![0, 0, 0, 0]),
                control: 0,
                guard_fail_ret: 0xDEAD_BEEF,
                events: Vec::new(),
            }
        }
    }

    impl BtStage92Backend for B {
        fn read_word(&mut self, address: u32) -> u32 {
            let value = match address {
                STAGE92_BT_GUARD_ADDR => self.guard_reads.pop_front().unwrap_or(self.guard),
                STAGE92_BT_SNAPSHOT_ADDR => self.snapshot,
                STAGE92_BT_CONTROL_WORD_ADDR => self.control,
                _ => panic!("unexpected read {address:#x}"),
            };
            self.events.push(("read", address, value));
            value
        }
        fn write_word(&mut self, address: u32, value: u32) {
            if address == STAGE92_BT_CONTROL_WORD_ADDR {
                self.control = value;
            }
            self.events.push(("write", address, value));
        }
        fn guard_fail_boundary(
            &mut self,
            current_r0: u32,
            current_r1: u32,
            saved_guard: u32,
            live_guard: u32,
        ) -> u32 {
            self.events.push(("guard_fail_r0", current_r0, current_r1));
            self.events.push(("guard_fail_guard", saved_guard, live_guard));
            self.guard_fail_ret
        }
    }

    impl BtStage90Backend for B {
        fn read_status_word(&mut self, address: u32) -> u32 {
            assert_eq!(address, STAGE90_BT_STATUS_WORD_ADDR);
            self.stage90_reads.pop_front().unwrap_or(0)
        }
    }

    impl BtStage91Backend for B {
        fn read_status_word(&mut self, address: u32) -> u32 {
            assert_eq!(address, STAGE91_BT_STATUS_WORD_ADDR);
            self.mode_reads.pop_front().unwrap_or(1 << 30)
        }
    }

    #[test]
    fn active_path_publishes_snapshot_bytes_in_little_endian_order() {
        let mut b = B::default();
        assert_eq!(bt_stage92_guarded_transfer(&mut b), 1);
        let pubs: Vec<u32> = b.events.iter()
            .filter(|e| e.0 == "write" && e.1 == STAGE92_BT_BYTE_PUBLISH_ADDR)
            .map(|e| e.2)
            .collect();
        assert_eq!(pubs, vec![0x11, 0x22, 0x33, 0x44]);
        assert_eq!(b.control & 8, 8);
    }

    #[test]
    fn bit4_gate_skips_transfer_and_returns_zero_when_guard_matches() {
        let mut b = B::default();
        b.mode_reads = VecDeque::from(vec![0x10]);
        assert_eq!(bt_stage92_guarded_transfer(&mut b), 0);
        assert!(!b.events.iter().any(|e| e.0 == "write"));
    }

    #[test]
    fn guard_mismatch_replaces_local_result_with_boundary_return() {
        let mut b = B::default();
        b.mode_reads = VecDeque::from(vec![0x10]);
        let saved = b.guard;
        b.guard_reads = VecDeque::from(vec![saved, saved ^ 1]);

        assert_eq!(bt_stage92_guarded_transfer(&mut b), 0xDEAD_BEEF);
        assert_eq!(
            b.events.iter().rev().take(2).copied().collect::<Vec<_>>(),
            vec![
                ("guard_fail_guard", saved, saved ^ 1),
                ("guard_fail_r0", 0, 0x10),
            ]
        );
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE92_CURRENT_BT_GUARDED_TRANSFER_ADDR, 0x171E1C);
        assert_eq!(STAGE92_BT_GUARD_ADDR, 0x200890);
        assert_eq!(STAGE92_BT_SNAPSHOT_ADDR, 0x222554);
        assert_eq!(STAGE92_BT_MODE_WORD_ADDR, 0x650310);
        assert_eq!(STAGE92_BT_BYTE_PUBLISH_ADDR, 0x650328);
        assert_eq!(STAGE92_BT_STAGE90_WORD_ADDR, 0x650318);
        assert_eq!(STAGE92_BT_CONTROL_WORD_ADDR, 0x650314);
        assert_eq!(STAGE92_BT_GUARD_FAIL_BOUNDARY, 0x94C0);
    }
}

/// Stage 93: clear ambient control bit 3 at current `0x171E8C`.
///
/// The exact 12-byte current body is byte-identical to public-legacy structural
/// `0x16DDDC`. The only observable R0 behavior is preservation of the incoming
/// value while one ambient dword is read-modify-written.
pub const STAGE93_CURRENT_BT_CLEAR_CONTROL_BIT3_ADDR: u32 = 0x0017_1E8C;
pub const STAGE93_BT_CONTROL_WORD_ADDR: u32 = 0x0065_0314;
pub const STAGE93_BT_CONTROL_BIT: u32 = 1 << 3;

pub trait BtStage93Backend {
    fn read_control_word(&mut self, address: u32) -> u32;
    fn write_control_word(&mut self, address: u32, value: u32);
}

/// Exact local model of current `0x171E8C`.
///
/// Firmware reads the ambient dword, clears only bit 3, writes it back, and
/// returns with incoming R0 untouched.
pub fn bt_stage93_clear_control_bit3<B: BtStage93Backend>(
    incoming_r0: u32,
    backend: &mut B,
) -> u32 {
    let old = backend.read_control_word(STAGE93_BT_CONTROL_WORD_ADDR);
    backend.write_control_word(
        STAGE93_BT_CONTROL_WORD_ADDR,
        old & !STAGE93_BT_CONTROL_BIT,
    );
    incoming_r0
}

#[cfg(test)]
mod stage93_tests {
    use super::*;

    #[derive(Default)]
    struct B {
        word: u32,
        reads: u32,
        writes: u32,
    }

    impl BtStage93Backend for B {
        fn read_control_word(&mut self, address: u32) -> u32 {
            assert_eq!(address, STAGE93_BT_CONTROL_WORD_ADDR);
            self.reads += 1;
            self.word
        }
        fn write_control_word(&mut self, address: u32, value: u32) {
            assert_eq!(address, STAGE93_BT_CONTROL_WORD_ADDR);
            self.writes += 1;
            self.word = value;
        }
    }

    #[test]
    fn clears_only_bit_three_and_preserves_r0() {
        let original = 0xA5A5_5A5A | STAGE93_BT_CONTROL_BIT;
        let mut b = B { word: original, ..Default::default() };
        assert_eq!(
            bt_stage93_clear_control_bit3(0xCAFE_BABE, &mut b),
            0xCAFE_BABE
        );
        assert_eq!(b.word, original & !STAGE93_BT_CONTROL_BIT);
        assert_eq!((b.reads, b.writes), (1, 1));
    }

    #[test]
    fn already_clear_bit_still_performs_exact_read_write() {
        let mut b = B { word: 0xFFFF_FFF7, ..Default::default() };
        assert_eq!(bt_stage93_clear_control_bit3(7, &mut b), 7);
        assert_eq!(b.word, 0xFFFF_FFF7);
        assert_eq!((b.reads, b.writes), (1, 1));
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE93_CURRENT_BT_CLEAR_CONTROL_BIT3_ADDR, 0x171E8C);
        assert_eq!(STAGE93_BT_CONTROL_WORD_ADDR, 0x650314);
        assert_eq!(STAGE93_BT_CONTROL_BIT, 8);
    }
}

/// Stage 94: extract ambient bits 16..18 at current `0x171E9C`.
///
/// The executable body is exactly 10 bytes. The following `NOP` is alignment
/// and is not part of the function. Public-legacy structural `0x16DDEC` is
/// byte-identical.
pub const STAGE94_CURRENT_BT_EXTRACT_BITS16_18_ADDR: u32 = 0x0017_1E9C;
pub const STAGE94_BT_SOURCE_WORD_ADDR: u32 = 0x0065_031C;
pub const STAGE94_BT_FIELD_SHIFT: u32 = 16;
pub const STAGE94_BT_FIELD_MASK: u32 = 0x7;

pub trait BtStage94Backend {
    fn read_source_word(&mut self, address: u32) -> u32;
}

/// Exact local model of current `0x171E9C`.
///
/// One dword is loaded and `UBFX R0,R0,#16,#3` becomes the function return.
pub fn bt_stage94_extract_bits16_18<B: BtStage94Backend>(
    backend: &mut B,
) -> u32 {
    (backend.read_source_word(STAGE94_BT_SOURCE_WORD_ADDR) >> STAGE94_BT_FIELD_SHIFT)
        & STAGE94_BT_FIELD_MASK
}

#[cfg(test)]
mod stage94_tests {
    use super::*;

    struct B {
        word: u32,
        reads: u32,
    }

    impl BtStage94Backend for B {
        fn read_source_word(&mut self, address: u32) -> u32 {
            assert_eq!(address, STAGE94_BT_SOURCE_WORD_ADDR);
            self.reads += 1;
            self.word
        }
    }

    #[test]
    fn extracts_only_bits_sixteen_through_eighteen() {
        let mut b = B { word: 0xFFFA_FFFF, reads: 0 };
        let expected = (b.word >> 16) & 7;
        assert_eq!(bt_stage94_extract_bits16_18(&mut b), expected);
        assert_eq!(b.reads, 1);
    }

    #[test]
    fn output_range_is_exactly_three_bits() {
        for field in 0u32..8 {
            let mut b = B { word: field << 16, reads: 0 };
            assert_eq!(bt_stage94_extract_bits16_18(&mut b), field);
        }
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE94_CURRENT_BT_EXTRACT_BITS16_18_ADDR, 0x171E9C);
        assert_eq!(STAGE94_BT_SOURCE_WORD_ADDR, 0x65031C);
        assert_eq!(STAGE94_BT_FIELD_SHIFT, 16);
        assert_eq!(STAGE94_BT_FIELD_MASK, 7);
    }
}

/// Stage 95: encode two incoming values then tail Stage 90 at current `0x171EAC`.
///
/// The executable body is exactly 22 bytes. The following `NOP` is alignment.
/// Public-legacy structural `0x16DDFC` is byte-identical.
pub const STAGE95_CURRENT_BT_ENCODE_AND_POLL_ADDR: u32 = 0x0017_1EAC;
pub const STAGE95_BT_VALUE_WORD_ADDR: u32 = 0x0065_0328;
pub const STAGE95_BT_STATUS_WORD_ADDR: u32 = 0x0065_0318;
pub const STAGE95_BT_INPUT_MASK: u32 = 0x0001_FF00;
pub const STAGE95_BT_STATUS_BASE: u32 = 0x8500_0000;
pub const STAGE95_BT_STAGE90_ADDR: u32 = STAGE90_CURRENT_BT_BOUNDED_STATUS_POLL_ADDR;

pub trait BtStage95Backend {
    fn write_stage95_word(&mut self, address: u32, value: u32);
}

/// Safe local model of current `0x171EAC`.
///
/// Incoming R0 is first published as a dword. Incoming R1 is shifted left by
/// eight, masked with `0x1FF00`, ORed with `0x85000000`, and published to the
/// Stage-90 status word. The final wide branch is a tail transfer to Stage 90,
/// so its return is the function return.
pub fn bt_stage95_encode_and_poll<B>(
    incoming_r0: u32,
    incoming_r1: u32,
    backend: &mut B,
) -> u32
where
    B: BtStage95Backend + BtStage90Backend,
{
    backend.write_stage95_word(STAGE95_BT_VALUE_WORD_ADDR, incoming_r0);
    let encoded = ((incoming_r1 << 8) & STAGE95_BT_INPUT_MASK)
        | STAGE95_BT_STATUS_BASE;
    backend.write_stage95_word(STAGE95_BT_STATUS_WORD_ADDR, encoded);
    bt_stage90_bounded_status_poll(backend)
}

#[cfg(test)]
mod stage95_tests {
    extern crate std;
    use super::*;
    use std::collections::VecDeque;
    use std::vec;
    use std::vec::Vec;

    struct B {
        status_reads: VecDeque<u32>,
        writes: Vec<(u32, u32)>,
    }

    impl BtStage95Backend for B {
        fn write_stage95_word(&mut self, address: u32, value: u32) {
            self.writes.push((address, value));
        }
    }

    impl BtStage90Backend for B {
        fn read_status_word(&mut self, address: u32) -> u32 {
            assert_eq!(address, STAGE90_BT_STATUS_WORD_ADDR);
            self.status_reads.pop_front().unwrap_or(0)
        }
    }

    #[test]
    fn publishes_r0_then_encoded_r1_in_binary_order() {
        let mut b = B {
            status_reads: VecDeque::from(vec![0]),
            writes: Vec::new(),
        };
        assert_eq!(bt_stage95_encode_and_poll(0x1234_5678, 0xABCD_01FF, &mut b), 1);
        assert_eq!(
            b.writes,
            vec![
                (STAGE95_BT_VALUE_WORD_ADDR, 0x1234_5678),
                (
                    STAGE95_BT_STATUS_WORD_ADDR,
                    ((0xABCD_01FFu32 << 8) & STAGE95_BT_INPUT_MASK)
                        | STAGE95_BT_STATUS_BASE,
                ),
            ]
        );
    }

    #[test]
    fn tail_return_is_exact_stage90_return() {
        let mut b = B {
            status_reads: VecDeque::from(vec![0xFFFF_FFFF; 100]),
            writes: Vec::new(),
        };
        assert_eq!(bt_stage95_encode_and_poll(9, 0, &mut b), 0);
    }

    #[test]
    fn high_r1_bits_are_discarded_by_shift_and_mask() {
        let mut b = B {
            status_reads: VecDeque::from(vec![0]),
            writes: Vec::new(),
        };
        let _ = bt_stage95_encode_and_poll(0, 0xFFFF_FFFF, &mut b);
        assert_eq!(
            b.writes[1],
            (
                STAGE95_BT_STATUS_WORD_ADDR,
                STAGE95_BT_STATUS_BASE | STAGE95_BT_INPUT_MASK,
            )
        );
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE95_CURRENT_BT_ENCODE_AND_POLL_ADDR, 0x171EAC);
        assert_eq!(STAGE95_BT_VALUE_WORD_ADDR, 0x650328);
        assert_eq!(STAGE95_BT_STATUS_WORD_ADDR, 0x650318);
        assert_eq!(STAGE95_BT_INPUT_MASK, 0x1FF00);
        assert_eq!(STAGE95_BT_STATUS_BASE, 0x85000000);
        assert_eq!(STAGE95_BT_STAGE90_ADDR, 0x171DEC);
    }
}

/// Stage 96: implementation closure for the Stage-25 raw mode-one target at `0x17218C`.
///
/// The routine installs a callback pointer and mode byte, runs a fixed opaque-boundary
/// sequence, then chooses between the Stage-25 mode-two target and a primary `(22, 19)` tail.
pub const STAGE96_CURRENT_BT_MODE1_TARGET_ADDR: u32 = 0x0017_218C;
pub const STAGE96_BT_CALLBACK_SLOT_ADDR: u32 = 0x0021_66D4;
pub const STAGE96_BT_CALLBACK_THUMB: u32 = 0x0017_1FD9;
pub const STAGE96_BT_MODE_ADDR: u32 = 0x0022_3064;
pub const STAGE96_BT_FALLBACK_GATE_BYTE_ADDR: u32 = 0x0020_CEDD;

pub const STAGE96_BT_BOUNDARY_86184: u32 = 0x0008_6184;
pub const STAGE96_BT_BOUNDARY_86370: u32 = 0x0008_6370;
pub const STAGE96_BT_BOUNDARY_89320: u32 = 0x0008_9320;
pub const STAGE96_BT_BOUNDARY_89398: u32 = 0x0008_9398;
pub const STAGE96_BT_BOUNDARY_860DC: u32 = 0x0008_60DC;
pub const STAGE96_BT_BOUNDARY_959C0: u32 = 0x0009_59C0;
pub const STAGE96_BT_GATE_BOUNDARY: u32 = 0x0003_386C;
pub const STAGE96_BT_OPTIONAL_BOUNDARY: u32 = 0x0002_DE80;
pub const STAGE96_BT_PRIMARY_TAIL: u32 = 0x0008_AEAC;
pub const STAGE96_BT_FALLBACK_MODE2_TARGET: u32 = STAGE25_BT_MODE2_TARGET_ADDR;

pub trait BtStage96Backend {
    fn write_callback_ptr(&mut self, address: u32, value: u32);
    fn write_mode_byte(&mut self, address: u32, value: u8);

    /// Only the register values explicitly established or preserved by local code are modeled.
    fn boundary_86184(&mut self, incoming_r0: u32, incoming_r1: u32, r2: u32, r3: u32) -> u32;
    fn boundary_86370(&mut self, live_r0: u32) -> u32;
    fn boundary_89320(&mut self, r0: u32) -> u32;
    fn boundary_89398(&mut self, r0: u32) -> u32;
    fn boundary_860dc(&mut self, r0: u32) -> u32;
    fn boundary_959c0(&mut self, live_r0: u32) -> u32;
    fn gate_boundary_3386c(&mut self, live_r0: u32) -> u32;

    fn read_fallback_gate_byte(&mut self, address: u32) -> u8;
    fn optional_boundary_2de80(&mut self, live_r0: u32) -> u32;

    fn primary_tail_8aeac(&mut self, r0: u32, r1: u32) -> u32;
    fn fallback_mode2_tail(&mut self, r0: u32) -> u32;
}

/// Safe source-level model of current `0x17218C`.
///
/// The model intentionally does not assign semantic names to the opaque runtime calls. It
/// preserves unconditional setup, the three explicit R0=0 resets, the live R0 chain into
/// `0x3386C`, short-circuiting of the fallback byte read, and both tail-return shapes.
pub fn bt_stage96_mode1_target<B: BtStage96Backend>(
    incoming_r0: u32,
    incoming_r1: u32,
    backend: &mut B,
) -> u32 {
    backend.write_callback_ptr(STAGE96_BT_CALLBACK_SLOT_ADDR, STAGE96_BT_CALLBACK_THUMB);
    backend.write_mode_byte(STAGE96_BT_MODE_ADDR, 2);

    let r0 = backend.boundary_86184(incoming_r0, incoming_r1, 2, STAGE96_BT_MODE_ADDR);
    let _ = backend.boundary_86370(r0);

    let _ = backend.boundary_89320(0);
    let _ = backend.boundary_89398(0);
    let r0 = backend.boundary_860dc(0);
    let r0 = backend.boundary_959c0(r0);
    let gate = backend.gate_boundary_3386c(r0);

    if gate == 0 && backend.read_fallback_gate_byte(STAGE96_BT_FALLBACK_GATE_BYTE_ADDR) == 0 {
        return backend.fallback_mode2_tail(0);
    }

    let _ = backend.optional_boundary_2de80(gate);
    backend.primary_tail_8aeac(22, 19)
}

#[cfg(test)]
mod stage96_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    struct B {
        gate: u32,
        gate_byte: u8,
        ret860dc: u32,
        ret959c0: u32,
        primary: u32,
        fallback: u32,
        events: Vec<(&'static str, u32, u32, u32, u32)>,
    }

    impl Default for B {
        fn default() -> Self {
            Self {
                gate: 0,
                gate_byte: 0,
                ret860dc: 0x60DC,
                ret959c0: 0x59C0,
                primary: 0x8AEAC,
                fallback: 0x1720E8,
                events: Vec::new(),
            }
        }
    }

    impl BtStage96Backend for B {
        fn write_callback_ptr(&mut self, a: u32, v: u32) {
            self.events.push(("callback", a, v, 0, 0));
        }
        fn write_mode_byte(&mut self, a: u32, v: u8) {
            self.events.push(("mode", a, v as u32, 0, 0));
        }
        fn boundary_86184(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32 {
            self.events.push(("86184", r0, r1, r2, r3));
            0x86184
        }
        fn boundary_86370(&mut self, r0: u32) -> u32 {
            self.events.push(("86370", r0, 0, 0, 0)); 0x86370
        }
        fn boundary_89320(&mut self, r0: u32) -> u32 {
            self.events.push(("89320", r0, 0, 0, 0)); 0x89320
        }
        fn boundary_89398(&mut self, r0: u32) -> u32 {
            self.events.push(("89398", r0, 0, 0, 0)); 0x89398
        }
        fn boundary_860dc(&mut self, r0: u32) -> u32 {
            self.events.push(("860dc", r0, 0, 0, 0)); self.ret860dc
        }
        fn boundary_959c0(&mut self, r0: u32) -> u32 {
            self.events.push(("959c0", r0, 0, 0, 0)); self.ret959c0
        }
        fn gate_boundary_3386c(&mut self, r0: u32) -> u32 {
            self.events.push(("3386c", r0, 0, 0, 0)); self.gate
        }
        fn read_fallback_gate_byte(&mut self, a: u32) -> u8 {
            self.events.push(("gate_byte", a, 0, 0, 0)); self.gate_byte
        }
        fn optional_boundary_2de80(&mut self, r0: u32) -> u32 {
            self.events.push(("2de80", r0, 0, 0, 0)); 0x2DE80
        }
        fn primary_tail_8aeac(&mut self, r0: u32, r1: u32) -> u32 {
            self.events.push(("primary_tail", r0, r1, 0, 0)); self.primary
        }
        fn fallback_mode2_tail(&mut self, r0: u32) -> u32 {
            self.events.push(("mode2_tail", r0, 0, 0, 0)); self.fallback
        }
    }

    #[test]
    fn setup_and_zero_resets_precede_gate_chain() {
        let mut b = B::default();
        let _ = bt_stage96_mode1_target(0xAA, 0xBB, &mut b);
        assert_eq!(b.events[0], ("callback", STAGE96_BT_CALLBACK_SLOT_ADDR, STAGE96_BT_CALLBACK_THUMB, 0, 0));
        assert_eq!(b.events[1], ("mode", STAGE96_BT_MODE_ADDR, 2, 0, 0));
        assert_eq!(b.events[2], ("86184", 0xAA, 0xBB, 2, STAGE96_BT_MODE_ADDR));
        assert_eq!(b.events[3], ("86370", 0x86184, 0, 0, 0));
        assert_eq!(b.events[4].0, "89320"); assert_eq!(b.events[4].1, 0);
        assert_eq!(b.events[5].0, "89398"); assert_eq!(b.events[5].1, 0);
        assert_eq!(b.events[6], ("860dc", 0, 0, 0, 0));
        assert_eq!(b.events[7], ("959c0", b.ret860dc, 0, 0, 0));
        assert_eq!(b.events[8], ("3386c", b.ret959c0, 0, 0, 0));
    }

    #[test]
    fn zero_gate_and_zero_byte_tail_to_mode2_with_zero_r0() {
        let mut b = B { gate: 0, gate_byte: 0, fallback: 0xCAFE, ..Default::default() };
        assert_eq!(bt_stage96_mode1_target(1, 2, &mut b), 0xCAFE);
        assert!(b.events.iter().any(|x|x.0=="gate_byte"));
        assert_eq!(b.events.last().copied(), Some(("mode2_tail", 0, 0, 0, 0)));
        assert!(!b.events.iter().any(|x|x.0=="2de80" || x.0=="primary_tail"));
    }

    #[test]
    fn nonzero_gate_skips_byte_read_and_primary_tail_uses_literals() {
        let mut b = B { gate: 7, primary: 0x12345678, ..Default::default() };
        assert_eq!(bt_stage96_mode1_target(1, 2, &mut b), 0x12345678);
        assert!(!b.events.iter().any(|x|x.0=="gate_byte"));
        assert!(b.events.iter().any(|x|*x==("2de80",7,0,0,0)));
        assert_eq!(b.events.last().copied(), Some(("primary_tail",22,19,0,0)));
    }

    #[test]
    fn zero_gate_nonzero_byte_calls_optional_with_zero_then_primary_tail() {
        let mut b = B { gate: 0, gate_byte: 1, primary: 9, ..Default::default() };
        assert_eq!(bt_stage96_mode1_target(0, 0, &mut b), 9);
        let g=b.events.iter().position(|x|x.0=="gate_byte").unwrap();
        let o=b.events.iter().position(|x|x.0=="2de80").unwrap();
        assert!(g<o);
        assert_eq!(b.events[o], ("2de80",0,0,0,0));
        assert_eq!(b.events.last().copied(), Some(("primary_tail",22,19,0,0)));
    }

    #[test]
    fn provenance_constants_close_stage25_mode_one_target() {
        assert_eq!(STAGE96_CURRENT_BT_MODE1_TARGET_ADDR, STAGE25_BT_MODE1_TARGET_ADDR);
        assert_eq!(STAGE96_BT_CALLBACK_SLOT_ADDR, 0x2166D4);
        assert_eq!(STAGE96_BT_CALLBACK_THUMB, 0x171FD9);
        assert_eq!(STAGE96_BT_MODE_ADDR, 0x223064);
        assert_eq!(STAGE96_BT_FALLBACK_GATE_BYTE_ADDR, 0x20CEDD);
        assert_eq!(STAGE96_BT_FALLBACK_MODE2_TARGET, 0x1720E8);
    }
}

/// Stage 97: implementation closure for the Stage-25 raw mode-two target at `0x1720E8`.
pub const STAGE97_CURRENT_BT_MODE2_TARGET_ADDR: u32 = 0x0017_20E8;
pub const STAGE97_BT_CONTEXT_ADDR: u32 = 0x0022_304C;
pub const STAGE97_BT_MODE_ADDR: u32 = 0x0022_3064;
pub const STAGE97_BT_DIRTY_ADDR: u32 = 0x0022_2FD0;
pub const STAGE97_BT_VALUE_ADDR: u32 = 0x0020_2FD4;
pub const STAGE97_BT_COMMAND_ADDR: u32 = 0x0022_2FD1;
pub const STAGE97_BT_WORD_A_ADDR: u32 = 0x0022_2080;
pub const STAGE97_BT_WORD_B_ADDR: u32 = 0x0022_2088;

pub trait BtStage97Backend {
    fn boundary_151bc(&mut self, context: u32) -> u32;
    fn write_mode_byte(&mut self, address: u32, value: u8);
    fn read_dirty_byte(&mut self, address: u32) -> u8;
    fn read_value_byte(&mut self, address: u32) -> u8;
    fn read_command_byte(&mut self, address: u32) -> u8;
    fn boundary_bacb4(&mut self, r0: u32, r1: u32, r2: u32) -> u32;
    fn boundary_bac58(&mut self, r0: u32, r1: u32) -> u32;
    fn boundary_72b24(&mut self, r0: u32) -> u32;
    fn boundary_71f08(&mut self, r0: u32) -> u32;
    fn boundary_780(&mut self, r0: u32) -> u32;
    fn boundary_89338(&mut self, r0: u32) -> u32;
    fn read_word(&mut self, address: u32) -> u32;
    fn boundary_89308(&mut self, r0: u32) -> u32;
    fn boundary_892f0(&mut self, r0: u32) -> u32;
    fn boundary_893c4(&mut self, r0: u32) -> u32;
    fn boundary_89368(&mut self, r0: u32) -> u32;
    fn boundary_44370(&mut self, live_r0: u32) -> u32;
    fn boundary_89320(&mut self, r0: u32) -> u32;
    fn boundary_89398(&mut self, r0: u32) -> u32;
    fn tail_860dc(&mut self, r0: u32) -> u32;
}

/// Safe source-level model of current `0x1720E8`.
///
/// Firmware ignores caller inputs locally: it loads the fixed context before its first call.
/// The value byte is intentionally reread after `0xBACB4`, and the return from the first
/// `0x780(1)` is saved across the reset sequence and supplied to the second `0x780`.
pub fn bt_stage97_mode2_target<B: BtStage97Backend>(backend: &mut B) -> u32 {
    let _ = backend.boundary_151bc(STAGE97_BT_CONTEXT_ADDR);
    backend.write_mode_byte(STAGE97_BT_MODE_ADDR, 3);

    if backend.read_dirty_byte(STAGE97_BT_DIRTY_ADDR) != 0 {
        let first = backend.read_value_byte(STAGE97_BT_VALUE_ADDR);
        let _ = backend.boundary_bacb4(first as u32, 0, 1);
        let second = backend.read_value_byte(STAGE97_BT_VALUE_ADDR);
        let command = backend.read_command_byte(STAGE97_BT_COMMAND_ADDR);
        let _ = backend.boundary_bac58(second as u32, command as u32);
    } else {
        let _ = backend.boundary_72b24(0);
    }

    let _ = backend.boundary_71f08(1);
    let saved = backend.boundary_780(1);

    let _ = backend.boundary_89338(0);
    let a = backend.read_word(STAGE97_BT_WORD_A_ADDR) & !1;
    let _ = backend.boundary_89308(a);
    let b = backend.read_word(STAGE97_BT_WORD_B_ADDR) & !1;
    let _ = backend.boundary_892f0(b);
    let _ = backend.boundary_893c4(0);
    let _ = backend.boundary_89368(0);

    let live = backend.boundary_780(saved);
    let _ = backend.boundary_44370(live);
    let _ = backend.boundary_89320(1);
    let _ = backend.boundary_89398(0);
    backend.tail_860dc(1)
}

#[cfg(test)]
mod stage97_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        dirty: u8,
        value: u8,
        command: u8,
        word_a: u32,
        word_b: u32,
        first_780: u32,
        second_780: u32,
        tail: u32,
        mutate_value: Option<u8>,
        n780: u8,
        events: Vec<(&'static str, u32, u32, u32)>,
    }
    impl BtStage97Backend for B {
        fn boundary_151bc(&mut self,c:u32)->u32{self.events.push(("151bc",c,0,0));0}
        fn write_mode_byte(&mut self,a:u32,v:u8){self.events.push(("mode",a,v as u32,0))}
        fn read_dirty_byte(&mut self,a:u32)->u8{self.events.push(("dirty",a,0,0));self.dirty}
        fn read_value_byte(&mut self,a:u32)->u8{self.events.push(("value",a,self.value as u32,0));self.value}
        fn read_command_byte(&mut self,a:u32)->u8{self.events.push(("command",a,self.command as u32,0));self.command}
        fn boundary_bacb4(&mut self,r0:u32,r1:u32,r2:u32)->u32{
            self.events.push(("bacb4",r0,r1,r2));if let Some(v)=self.mutate_value{self.value=v;}0
        }
        fn boundary_bac58(&mut self,r0:u32,r1:u32)->u32{self.events.push(("bac58",r0,r1,0));0}
        fn boundary_72b24(&mut self,r0:u32)->u32{self.events.push(("72b24",r0,0,0));0}
        fn boundary_71f08(&mut self,r0:u32)->u32{self.events.push(("71f08",r0,0,0));0}
        fn boundary_780(&mut self,r0:u32)->u32{
            self.events.push(("780",r0,0,0));self.n780+=1;
            if self.n780==1{self.first_780}else{self.second_780}
        }
        fn boundary_89338(&mut self,r0:u32)->u32{self.events.push(("89338",r0,0,0));0}
        fn read_word(&mut self,a:u32)->u32{self.events.push(("word",a,0,0));if a==STAGE97_BT_WORD_A_ADDR{self.word_a}else{self.word_b}}
        fn boundary_89308(&mut self,r0:u32)->u32{self.events.push(("89308",r0,0,0));0}
        fn boundary_892f0(&mut self,r0:u32)->u32{self.events.push(("892f0",r0,0,0));0}
        fn boundary_893c4(&mut self,r0:u32)->u32{self.events.push(("893c4",r0,0,0));0}
        fn boundary_89368(&mut self,r0:u32)->u32{self.events.push(("89368",r0,0,0));0}
        fn boundary_44370(&mut self,r0:u32)->u32{self.events.push(("44370",r0,0,0));0}
        fn boundary_89320(&mut self,r0:u32)->u32{self.events.push(("89320",r0,0,0));0}
        fn boundary_89398(&mut self,r0:u32)->u32{self.events.push(("89398",r0,0,0));0}
        fn tail_860dc(&mut self,r0:u32)->u32{self.events.push(("860dc",r0,0,0));self.tail}
    }

    #[test]
    fn zero_dirty_uses_72b24_and_skips_pair_boundaries() {
        let mut b=B{tail:9,..Default::default()};
        assert_eq!(bt_stage97_mode2_target(&mut b),9);
        assert!(b.events.iter().any(|x|*x==("72b24",0,0,0)));
        assert!(!b.events.iter().any(|x|x.0=="bacb4"||x.0=="bac58"));
    }

    #[test]
    fn dirty_path_rereads_value_after_first_boundary_mutation() {
        let mut b=B{dirty:1,value:4,command:7,mutate_value:Some(9),..Default::default()};
        let _=bt_stage97_mode2_target(&mut b);
        let vals:Vec<u32>=b.events.iter().filter(|x|x.0=="value").map(|x|x.2).collect();
        assert_eq!(vals,[4,9]);
        assert!(b.events.iter().any(|x|*x==("bacb4",4,0,1)));
        assert!(b.events.iter().any(|x|*x==("bac58",9,7,0)));
    }

    #[test]
    fn first_780_result_survives_reset_sequence_into_second_780() {
        let mut b=B{first_780:0x1234,second_780:0x5678,..Default::default()};
        let _=bt_stage97_mode2_target(&mut b);
        let calls:Vec<u32>=b.events.iter().filter(|x|x.0=="780").map(|x|x.1).collect();
        assert_eq!(calls,[1,0x1234]);
        assert!(b.events.iter().any(|x|*x==("44370",0x5678,0,0)));
    }

    #[test]
    fn word_calls_clear_only_bit_zero() {
        let mut b=B{word_a:0xFFFF_FFFF,word_b:0x1234_5679,..Default::default()};
        let _=bt_stage97_mode2_target(&mut b);
        assert!(b.events.iter().any(|x|*x==("89308",0xFFFF_FFFE,0,0)));
        assert!(b.events.iter().any(|x|*x==("892f0",0x1234_5678,0,0)));
    }

    #[test]
    fn final_boundaries_use_literal_r0_and_tail_return_is_final() {
        let mut b=B{tail:0xDEAD_BEEF,..Default::default()};
        assert_eq!(bt_stage97_mode2_target(&mut b),0xDEAD_BEEF);
        let n=b.events.len();
        assert_eq!(&b.events[n-3..],&[("89320",1,0,0),("89398",0,0,0),("860dc",1,0,0)]);
    }

    #[test]
    fn provenance_constants_close_stage25_mode_two_target() {
        assert_eq!(STAGE97_CURRENT_BT_MODE2_TARGET_ADDR,STAGE25_BT_MODE2_TARGET_ADDR);
        assert_eq!(STAGE97_BT_CONTEXT_ADDR,0x22304C);
        assert_eq!(STAGE97_BT_MODE_ADDR,0x223064);
        assert_eq!(STAGE97_BT_DIRTY_ADDR,0x222FD0);
        assert_eq!(STAGE97_BT_VALUE_ADDR,0x202FD4);
        assert_eq!(STAGE97_BT_COMMAND_ADDR,0x222FD1);
        assert_eq!(STAGE97_BT_WORD_A_ADDR,0x222080);
        assert_eq!(STAGE97_BT_WORD_B_ADDR,0x222088);
    }
}

/// Stage 98: current active object/register orchestrator at `0x172320`.
///
/// The exact 142-byte current body is relocation-normalized against the public
/// legacy structural counterpart at `0x16E210`. The safe model preserves the
/// mode gates, exact field reads, Stage-23 lookup shape, stack-byte alias seen
/// by the still-opaque `0x172258` dependency, registration/finalize ordering,
/// ignored returns, and final zero result. Compiler stack-canary plumbing is
/// recorded as provenance but intentionally omitted from the safe model.
pub const STAGE98_CURRENT_BT_ACTIVE_DISPATCH_ADDR: u32 = 0x0017_2320;
pub const STAGE98_PUBLIC_LEGACY_STRUCTURAL_ADDR: u32 = 0x0016_E210;
pub const STAGE98_BT_MODE_ADDR: u32 = 0x0022_3064;
pub const STAGE98_BT_CONTEXT_ADDR: u32 = 0x0022_3068;
pub const STAGE98_BT_CALLBACK_THUMB: u32 = 0x0017_1FDD;
pub const STAGE98_BT_GUARD_WORD_ADDR: u32 = 0x0020_0890;
pub const STAGE98_BT_STACK_GUARD_FAIL: u32 = ROM_STACK_GUARD_FAIL_ADDR;
pub const STAGE98_BT_FIND_MATCHING_SLOT_ADDR: u32 = STAGE23_CURRENT_BT_FIND_MATCHING_SLOT_ADDR;
pub const STAGE98_BT_PREPARE_BOUNDARY: u32 = 0x0017_2258;
pub const STAGE98_BT_REGISTER_BOUNDARY: u32 = 0x0001_51FE;
pub const STAGE98_BT_FINALIZE_BOUNDARY: u32 = 0x0001_5180;
pub const STAGE98_BT_TOGGLE_COMMAND_ADDR: u32 = STAGE29_CURRENT_BT_TOGGLE_COMMAND_DISPATCH_ADDR;
pub const STAGE98_BT_REQUIRED_HEADER: u16 = 13;
pub const STAGE98_BT_NOT_FOUND_INDEX: u32 = 8;
pub const STAGE98_BT_INTERVAL_SCALE: u32 = 0x30D4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage98ObjectState {
    pub word0: u16,
    pub byte2: u8,
    pub byte3: u8,
    pub dword12: u32,
}

pub trait BtStage98Backend {
    fn read_mode(&mut self) -> u8;
    fn find_matching_slot(&mut self, tag: u8, payload_ptr: u32) -> u32;
    fn prepare(&mut self, payload_plus6: u32, selector: u8, local_byte: &mut u8) -> u32;
    fn write_mode(&mut self, value: u8);
    fn register(&mut self, context: u32, callback_thumb: u32, zero: u32, interval: u32) -> u32;
    fn finalize(&mut self, context: u32, interval: u32) -> u32;
    fn toggle_command(&mut self, zero_r0: u32, zero_r1: u32) -> u32;
}

/// Safe source-level model of current `0x172320`.
///
/// `incoming_r0` is retained separately because firmware's stack scratch byte
/// at `sp+3` initially aliases bits 31:24 of saved incoming R0. The opaque
/// prepare helper may overwrite that byte through its pointer argument.
pub fn bt_stage98_active_dispatch<B: BtStage98Backend>(
    incoming_r0: u32,
    incoming_r1: u32,
    object: &BtStage98ObjectState,
    backend: &mut B,
) -> u32 {
    let mode = backend.read_mode();

    if mode <= 2 {
        return u32::from(incoming_r1 as u8);
    }
    if mode == 4 {
        return 0;
    }

    if object.word0 != STAGE98_BT_REQUIRED_HEADER {
        return 0;
    }

    let packed = object.byte2;
    let low_nibble = packed & 0x0F;
    if low_nibble != 0 {
        return 0;
    }

    let payload = object.dword12;
    let selector = object.byte3;
    let tag = packed >> 6;
    if backend.find_matching_slot(tag, payload) == STAGE98_BT_NOT_FOUND_INDEX {
        return 0;
    }

    // Firmware passes `sp+3`; before the helper touches it this byte is the
    // high byte of the saved incoming R0 scratch slot.
    let mut local_byte = (incoming_r0 >> 24) as u8;
    if backend.prepare(payload.wrapping_add(6), selector, &mut local_byte) == 0 {
        return 0;
    }

    backend.write_mode(4);

    let interval = STAGE98_BT_INTERVAL_SCALE.wrapping_mul(u32::from(local_byte));
    let _ = backend.register(
        STAGE98_BT_CONTEXT_ADDR,
        STAGE98_BT_CALLBACK_THUMB,
        0,
        interval,
    );
    let _ = backend.finalize(STAGE98_BT_CONTEXT_ADDR, interval);

    // Current helper receives explicit zero in R0/R1; its return is ignored.
    let _ = backend.toggle_command(0, 0);

    0
}

#[cfg(test)]
mod stage98_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        mode: u8,
        lookup: u32,
        prepare_return: u32,
        prepare_write: Option<u8>,
        calls: Vec<(&'static str, u32, u32, u32, u32)>,
        seen_local: Option<u8>,
    }

    impl BtStage98Backend for B {
        fn read_mode(&mut self) -> u8 {
            self.calls.push(("mode", 0, 0, 0, 0));
            self.mode
        }
        fn find_matching_slot(&mut self, tag: u8, payload_ptr: u32) -> u32 {
            self.calls.push(("lookup", u32::from(tag), payload_ptr, 0, 0));
            self.lookup
        }
        fn prepare(&mut self, payload_plus6: u32, selector: u8, local_byte: &mut u8) -> u32 {
            self.seen_local = Some(*local_byte);
            self.calls.push(("prepare", payload_plus6, u32::from(selector), u32::from(*local_byte), 0));
            if let Some(v) = self.prepare_write { *local_byte = v; }
            self.prepare_return
        }
        fn write_mode(&mut self, value: u8) {
            self.calls.push(("write_mode", u32::from(value), 0, 0, 0));
            self.mode = value;
        }
        fn register(&mut self, context: u32, callback_thumb: u32, zero: u32, interval: u32) -> u32 {
            self.calls.push(("register", context, callback_thumb, zero, interval));
            0xAAAA_AAAA
        }
        fn finalize(&mut self, context: u32, interval: u32) -> u32 {
            self.calls.push(("finalize", context, interval, 0, 0));
            0xBBBB_BBBB
        }
        fn toggle_command(&mut self, zero_r0: u32, zero_r1: u32) -> u32 {
            self.calls.push(("toggle", zero_r0, zero_r1, 0, 0));
            0xCCCC_CCCC
        }
    }

    fn object() -> BtStage98ObjectState {
        BtStage98ObjectState { word0: 13, byte2: 0x80, byte3: 7, dword12: 0x1000 }
    }

    #[test]
    fn modes_zero_through_two_return_only_low_byte_of_incoming_r1() {
        for mode in 0..=2 {
            let mut b = B { mode, ..Default::default() };
            assert_eq!(bt_stage98_active_dispatch(0xAABB_CCDD, 0x1234_56FE, &object(), &mut b), 0xFE);
            assert_eq!(b.calls, [("mode", 0, 0, 0, 0)]);
        }
    }

    #[test]
    fn mode_four_is_strict_zero_return() {
        let mut b = B { mode: 4, ..Default::default() };
        assert_eq!(bt_stage98_active_dispatch(1, 2, &object(), &mut b), 0);
        assert_eq!(b.calls, [("mode", 0, 0, 0, 0)]);
    }

    #[test]
    fn active_gates_preserve_lookup_tag_and_not_found_shape() {
        let mut b = B { mode: 3, lookup: 8, ..Default::default() };
        assert_eq!(bt_stage98_active_dispatch(0, 0, &object(), &mut b), 0);
        assert_eq!(b.calls[1], ("lookup", 2, 0x1000, 0, 0));

        let mut bad = object();
        bad.byte2 = 0x81;
        let mut b = B { mode: 3, ..Default::default() };
        assert_eq!(bt_stage98_active_dispatch(0, 0, &bad, &mut b), 0);
        assert_eq!(b.calls.len(), 1);
    }

    #[test]
    fn prepare_sees_high_byte_of_saved_incoming_r0_before_any_write() {
        let mut b = B { mode: 3, lookup: 2, prepare_return: 0, ..Default::default() };
        assert_eq!(bt_stage98_active_dispatch(0xA512_3456, 0, &object(), &mut b), 0);
        assert_eq!(b.seen_local, Some(0xA5));
        assert_eq!(b.calls[2], ("prepare", 0x1006, 7, 0xA5, 0));
    }

    #[test]
    fn success_uses_mutated_local_for_both_interval_calls_then_ignores_toggle_return() {
        let mut b = B {
            mode: 3,
            lookup: 1,
            prepare_return: 9,
            prepare_write: Some(3),
            ..Default::default()
        };
        assert_eq!(bt_stage98_active_dispatch(0x4400_0000, 0, &object(), &mut b), 0);
        let interval = STAGE98_BT_INTERVAL_SCALE * 3;
        assert_eq!(b.calls, [
            ("mode", 0, 0, 0, 0),
            ("lookup", 2, 0x1000, 0, 0),
            ("prepare", 0x1006, 7, 0x44, 0),
            ("write_mode", 4, 0, 0, 0),
            ("register", STAGE98_BT_CONTEXT_ADDR, STAGE98_BT_CALLBACK_THUMB, 0, interval),
            ("finalize", STAGE98_BT_CONTEXT_ADDR, interval, 0, 0),
            ("toggle", 0, 0, 0, 0),
        ]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE98_CURRENT_BT_ACTIVE_DISPATCH_ADDR, 0x172320);
        assert_eq!(STAGE98_PUBLIC_LEGACY_STRUCTURAL_ADDR, 0x16E210);
        assert_eq!(STAGE98_BT_FIND_MATCHING_SLOT_ADDR, 0x17208C);
        assert_eq!(STAGE98_BT_PREPARE_BOUNDARY, 0x172258);
        assert_eq!(STAGE98_BT_REGISTER_BOUNDARY, 0x151FE);
        assert_eq!(STAGE98_BT_FINALIZE_BOUNDARY, 0x15180);
        assert_eq!(STAGE98_BT_TOGGLE_COMMAND_ADDR, 0x171FDC);
        assert_eq!(STAGE98_BT_STACK_GUARD_FAIL, 0x94C0);
    }
}

/// Stage 99: current record-mask walker at `0x172258`.
///
/// The exact 186-byte current body has no promoted public-legacy normalized
/// counterpart. The model preserves signed span clamping, the post-iteration
/// UXTB remaining counter, independent rereads of the ambient mask count, the
/// scratch-mask pass, compare gate, and last-match output behavior. Compiler
/// stack-canary plumbing is provenance only.
pub const STAGE99_CURRENT_BT_RECORD_MASK_WALK_ADDR: u32 = 0x0017_2258;
pub const STAGE99_BT_COPY_BOUNDARY: u32 = 0x0000_3DB4;
pub const STAGE99_BT_COMPARE_BOUNDARY: u32 = 0x000F_8CAC;
pub const STAGE99_BT_PAIR_LOOKUP_ADDR: u32 = STAGE25_CURRENT_BT_PAIR_CONFIG_LOOKUP_ADDR;
pub const STAGE99_BT_GUARD_WORD_ADDR: u32 = 0x0020_0890;
pub const STAGE99_BT_STACK_GUARD_FAIL: u32 = ROM_STACK_GUARD_FAIL_ADDR;
pub const STAGE99_BT_MASK_BASE_ADDR: u32 = 0x0022_300E;
pub const STAGE99_BT_REFERENCE_ADDR: u32 = 0x0022_300F;

pub trait BtStage99Backend {
    fn read_input_byte(&mut self, address: u32) -> u8;
    fn read_mask_count(&mut self) -> u8;

    /// Current `0x3DB4(scratch, source, len)`. The return is ignored.
    fn copy_to_scratch(&mut self, source: u32, len: u32) -> u32;

    /// Read one byte relative to current mask base `0x22300E`.
    fn read_mask_byte(&mut self, offset: u32) -> u8;

    fn read_scratch_byte(&mut self, index: u32) -> u8;
    fn write_scratch_byte(&mut self, index: u32, value: u8);

    /// Current memcmp-like `0xF8CAC(scratch, 0x22300F, len)`.
    fn compare_scratch(&mut self, reference: u32, len: u32) -> u32;

    /// Current Stage-25 lookup receives R0=record+2. Firmware also computes a
    /// caller R1 byte immediately before the call even though the recovered
    /// callee does not consume it; keep it visible for exact call-site ABI.
    fn pair_lookup(&mut self, key_ptr: u32, computed_r1: u8) -> u8;
}

/// Safe source-level model of current `0x172258`.
pub fn bt_stage99_record_mask_walk<B: BtStage99Backend>(
    input_ptr: u32,
    input_len: u32,
    output: &mut u8,
    backend: &mut B,
) -> u32 {
    let last = input_len.wrapping_sub(1);
    let mut ptr = input_ptr;
    let mut remaining = input_len;
    let mut found = 0u32;

    while remaining != 0 {
        let mut span = u32::from(backend.read_input_byte(ptr));

        // Current CMP + IT GE uses signed GE, despite span originating as u8.
        if (span as i32) >= (last as i32) {
            span = last;
        }

        let span8 = span as u8;

        if backend.read_input_byte(ptr.wrapping_add(1)) == 0xFF {
            let count_before_copy = backend.read_mask_count();

            if span8 >= count_before_copy {
                let source = ptr.wrapping_add(2);
                let _ = backend.copy_to_scratch(source, u32::from(count_before_copy));

                // Reread after copy. Firmware can mask more scratch bytes than
                // were copied when this value grows; the backend exposes those
                // pre-existing scratch bytes rather than inventing zeroes.
                let count_for_mask = backend.read_mask_count();
                let mut i = 0u32;
                while i < u32::from(count_for_mask) {
                    let mask = backend.read_mask_byte(
                        u32::from(count_before_copy).wrapping_add(i).wrapping_add(1),
                    );
                    let value = backend.read_scratch_byte(i) & mask;
                    backend.write_scratch_byte(i, value);
                    i += 1;
                }

                if backend.compare_scratch(
                    STAGE99_BT_REFERENCE_ADDR,
                    u32::from(count_for_mask),
                ) == 0 {
                    let count_for_delta = backend.read_mask_count();
                    let computed_r1 = span8.wrapping_sub(count_for_delta);
                    let value = backend.pair_lookup(source, computed_r1);
                    found = 1;
                    *output = value;
                }
            }
        }

        // Binary sequence is `remaining += ~span8; UXTB remaining`.
        remaining = u32::from(
            remaining
                .wrapping_sub(u32::from(span8))
                .wrapping_sub(1) as u8,
        );

        // `span` is UXTB'd before the add, but the +1 itself is 32-bit:
        // span8=255 advances by 256, not by zero.
        ptr = ptr.wrapping_add(u32::from(span8).wrapping_add(1));
    }

    found
}

#[cfg(test)]
mod stage99_tests {
    extern crate std;
    use super::*;
    use std::vec;
    use std::vec::Vec;

    struct B {
        base: u32,
        input: [u8; 512],
        counts: Vec<u8>,
        ci: usize,
        mask: [u8; 256],
        scratch: [u8; 256],
        compare: u32,
        pair_value: u8,
        calls: Vec<(&'static str, u32, u32)>,
    }

    impl B {
        fn new() -> Self {
            Self {
                base: 0x1000,
                input: [0; 512],
                counts: Vec::new(),
                ci: 0,
                mask: [0xFF; 256],
                scratch: [0; 256],
                compare: 1,
                pair_value: 0,
                calls: Vec::new(),
            }
        }
    }

    impl BtStage99Backend for B {
        fn read_input_byte(&mut self, address: u32) -> u8 {
            self.calls.push(("read", address, 0));
            self.input[address.wrapping_sub(self.base) as usize]
        }
        fn read_mask_count(&mut self) -> u8 {
            let v = self.counts[self.ci];
            self.ci += 1;
            self.calls.push(("count", u32::from(v), 0));
            v
        }
        fn copy_to_scratch(&mut self, source: u32, len: u32) -> u32 {
            self.calls.push(("copy", source, len));
            let mut i = 0;
            while i < len {
                self.scratch[i as usize] =
                    self.input[source.wrapping_sub(self.base).wrapping_add(i) as usize];
                i += 1;
            }
            0xDEAD_BEEF
        }
        fn read_mask_byte(&mut self, offset: u32) -> u8 {
            self.calls.push(("mask", offset, 0));
            self.mask[offset as usize]
        }
        fn read_scratch_byte(&mut self, index: u32) -> u8 {
            self.scratch[index as usize]
        }
        fn write_scratch_byte(&mut self, index: u32, value: u8) {
            self.calls.push(("scratch", index, u32::from(value)));
            self.scratch[index as usize] = value;
        }
        fn compare_scratch(&mut self, reference: u32, len: u32) -> u32 {
            self.calls.push(("compare", reference, len));
            self.compare
        }
        fn pair_lookup(&mut self, key_ptr: u32, computed_r1: u8) -> u8 {
            self.calls.push(("pair", key_ptr, u32::from(computed_r1)));
            self.pair_value
        }
    }

    #[test]
    fn zero_length_is_strict_no_access_return_zero() {
        let mut b = B::new();
        let mut out = 7;
        assert_eq!(bt_stage99_record_mask_walk(b.base, 0, &mut out, &mut b), 0);
        assert_eq!(out, 7);
        assert!(b.calls.is_empty());
    }

    #[test]
    fn signed_clamp_and_uxth_like_byte_remaining_advance_are_exact() {
        let mut b = B::new();
        b.input[0] = 10;
        b.input[1] = 0;
        let mut out = 0;
        assert_eq!(bt_stage99_record_mask_walk(b.base, 3, &mut out, &mut b), 0);
        // span is clamped to last=2, so one iteration consumes all three bytes.
        assert_eq!(b.calls, [("read", 0x1000, 0), ("read", 0x1001, 0)]);
    }

    #[test]
    fn special_record_rereads_counts_masks_scratch_and_publishes_pair_value() {
        let mut b = B::new();
        b.input[0] = 3;
        b.input[1] = 0xFF;
        b.input[2] = 0x0F;
        b.input[3] = 0x0A;
        b.counts = vec![2, 2, 1];
        b.mask[3] = 0x0C;
        b.mask[4] = 0xFF;
        b.compare = 0;
        b.pair_value = 0x5A;
        let mut out = 0;
        assert_eq!(bt_stage99_record_mask_walk(b.base, 4, &mut out, &mut b), 1);
        assert_eq!(out, 0x5A);
        assert_eq!(b.scratch[0], 0x0C);
        assert_eq!(b.scratch[1], 0x0A);
        assert!(b.calls.contains(&("pair", 0x1002, 2)));
    }

    #[test]
    fn grown_second_count_masks_preexisting_scratch_beyond_copy_length() {
        let mut b = B::new();
        b.input[0] = 2;
        b.input[1] = 0xFF;
        b.input[2] = 0xF0;
        b.counts = vec![1, 2];
        b.scratch[1] = 0xAA; // models pre-existing stack scratch byte
        b.mask[2] = 0x0F;
        b.mask[3] = 0xF0;
        b.compare = 1;
        let mut out = 0;
        assert_eq!(bt_stage99_record_mask_walk(b.base, 3, &mut out, &mut b), 0);
        assert_eq!(b.scratch[0], 0);
        assert_eq!(b.scratch[1], 0xA0);
    }

    #[test]
    fn later_matching_record_overwrites_output_but_found_remains_one() {
        let mut b = B::new();
        // Two 2-byte records, both special, count zero avoids scratch reads.
        b.input[0] = 1; b.input[1] = 0xFF;
        b.input[2] = 1; b.input[3] = 0xFF;
        b.counts = vec![0, 0, 0, 0, 0, 0];
        b.compare = 0;
        b.pair_value = 9;
        let mut out = 0;
        assert_eq!(bt_stage99_record_mask_walk(b.base, 4, &mut out, &mut b), 1);
        assert_eq!(out, 9);
        assert_eq!(b.calls.iter().filter(|x|x.0=="pair").count(), 2);
    }

    #[test]
    fn provenance_constants_are_current_only() {
        assert_eq!(STAGE99_CURRENT_BT_RECORD_MASK_WALK_ADDR, 0x172258);
        assert_eq!(STAGE99_BT_COPY_BOUNDARY, 0x3DB4);
        assert_eq!(STAGE99_BT_COMPARE_BOUNDARY, 0xF8CAC);
        assert_eq!(STAGE99_BT_PAIR_LOOKUP_ADDR, 0x172220);
        assert_eq!(STAGE99_BT_MASK_BASE_ADDR, 0x22300E);
        assert_eq!(STAGE99_BT_REFERENCE_ADDR, 0x22300F);
    }
}


/// Stage 100: adjacent current wrappers that preserve R0..R3 across one local helper.
///
/// Current entries 0x16D5BA and 0x16D5CC each save incoming R0..R3/LR, move ambient
/// R4 into R0, call the same local helper at 0x16D5A4, restore the saved registers,
/// then tail-transfer to a distinct stable boundary. The helper return is therefore
/// not observable through R0 on either tail.
pub const STAGE100_CURRENT_BT_WRAPPER_A_ADDR: u32 = 0x0016_D5BA;
pub const STAGE100_CURRENT_BT_WRAPPER_B_ADDR: u32 = 0x0016_D5CC;
pub const STAGE100_BT_LOCAL_HELPER_ADDR: u32 = 0x0016_D5A4;
pub const STAGE100_BT_TAIL_A_ADDR: u32 = 0x0003_1C6C;
pub const STAGE100_BT_TAIL_B_ADDR: u32 = 0x0003_235E;

pub trait BtStage100Backend {
    /// Opaque local helper called as `0x16D5A4(ambient_r4)`. Its return is discarded
    /// when firmware restores saved R0..R3.
    fn local_helper(&mut self, ambient_r4: u32) -> u32;
    fn tail_a(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;
    fn tail_b(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;
}

pub fn bt_stage100_wrapper_a<B: BtStage100Backend>(
    r0: u32, r1: u32, r2: u32, r3: u32, ambient_r4: u32, backend: &mut B,
) -> u32 {
    let _ = backend.local_helper(ambient_r4);
    backend.tail_a(r0, r1, r2, r3)
}

pub fn bt_stage100_wrapper_b<B: BtStage100Backend>(
    r0: u32, r1: u32, r2: u32, r3: u32, ambient_r4: u32, backend: &mut B,
) -> u32 {
    let _ = backend.local_helper(ambient_r4);
    backend.tail_b(r0, r1, r2, r3)
}

#[cfg(test)]
mod stage100_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[derive(Default)]
    struct B { events: Vec<(u32,u32,u32,u32,u32)>, helper_ret: u32, tail_ret: u32 }
    impl BtStage100Backend for B {
        fn local_helper(&mut self, ambient_r4: u32) -> u32 {
            self.events.push((1,ambient_r4,0,0,0)); self.helper_ret
        }
        fn tail_a(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->u32 {
            self.events.push((2,r0,r1,r2,r3)); self.tail_ret
        }
        fn tail_b(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->u32 {
            self.events.push((3,r0,r1,r2,r3)); self.tail_ret
        }
    }

    #[test]
    fn wrapper_a_discards_helper_return_and_restores_all_four_argument_registers() {
        let mut b=B{helper_ret:0xDEAD_BEEF,tail_ret:0xAABB_CCDD,..Default::default()};
        assert_eq!(bt_stage100_wrapper_a(1,2,3,4,0x55,&mut b),0xAABB_CCDD);
        assert_eq!(b.events,[(1,0x55,0,0,0),(2,1,2,3,4)]);
    }

    #[test]
    fn wrapper_b_has_the_same_prepare_shape_but_distinct_tail() {
        let mut b=B{helper_ret:7,tail_ret:9,..Default::default()};
        assert_eq!(bt_stage100_wrapper_b(10,11,12,13,14,&mut b),9);
        assert_eq!(b.events,[(1,14,0,0,0),(3,10,11,12,13)]);
    }

    #[test]
    fn zero_register_values_are_forwarded_without_local_guards() {
        let mut b=B{helper_ret:1,tail_ret:2,..Default::default()};
        assert_eq!(bt_stage100_wrapper_a(0,0,0,0,0,&mut b),2);
        assert_eq!(b.events,[(1,0,0,0,0),(2,0,0,0,0)]);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE100_CURRENT_BT_WRAPPER_A_ADDR,0x16D5BA);
        assert_eq!(STAGE100_CURRENT_BT_WRAPPER_B_ADDR,0x16D5CC);
        assert_eq!(STAGE100_BT_LOCAL_HELPER_ADDR,0x16D5A4);
        assert_eq!(STAGE100_BT_TAIL_A_ADDR,0x31C6C);
        assert_eq!(STAGE100_BT_TAIL_B_ADDR,0x3235E);
    }
}


/// Stage 101: current bit-relation tail helper at `0x16D5A4`.
///
/// Current firmware reads object halfword +0x26 first, then object byte +0x1D.
/// It compares halfword bit4 with byte bit7. A mismatch returns the incoming
/// object token unchanged and does not read object word0. A match reads word0
/// and tail-transfers it to opaque current `0x32720`.
pub const STAGE101_CURRENT_BT_BIT_RELATION_TAIL_ADDR: u32 = 0x0016_D5A4;
pub const STAGE101_BT_TAIL_BOUNDARY: u32 = 0x0003_2720;

pub trait BtStage101Backend {
    fn read_halfword38(&mut self, object: u32) -> u16;
    fn read_byte29(&mut self, object: u32) -> u8;
    fn read_word0(&mut self, object: u32) -> u32;
    fn tail(&mut self, word0: u32) -> u32;
}

pub fn bt_stage101_bit_relation_tail<B: BtStage101Backend>(
    object: u32,
    backend: &mut B,
) -> u32 {
    let halfword38 = backend.read_halfword38(object);
    let byte29 = backend.read_byte29(object);
    let bit4 = (halfword38 >> 4) & 1;
    let bit7 = u16::from(byte29 >> 7);
    if bit4 != bit7 {
        return object;
    }
    let word0 = backend.read_word0(object);
    backend.tail(word0)
}

#[cfg(test)]
mod stage101_tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    struct B {
        halfword38: u16,
        byte29: u8,
        word0: u32,
        tail_ret: u32,
        calls: Vec<(&'static str, u32)>,
    }

    impl BtStage101Backend for B {
        fn read_halfword38(&mut self, object: u32) -> u16 {
            self.calls.push(("halfword38", object));
            self.halfword38
        }
        fn read_byte29(&mut self, object: u32) -> u8 {
            self.calls.push(("byte29", object));
            self.byte29
        }
        fn read_word0(&mut self, object: u32) -> u32 {
            self.calls.push(("word0", object));
            self.word0
        }
        fn tail(&mut self, word0: u32) -> u32 {
            self.calls.push(("tail", word0));
            self.tail_ret
        }
    }

    fn backend(halfword38: u16, byte29: u8) -> B {
        B {
            halfword38,
            byte29,
            word0: 0x1122_3344,
            tail_ret: 0x5566_7788,
            calls: Vec::new(),
        }
    }

    #[test]
    fn mismatch_returns_object_and_does_not_read_word0_or_tail() {
        let mut b = backend(0x0000, 0x80);
        assert_eq!(bt_stage101_bit_relation_tail(0xAABB_CCDD, &mut b), 0xAABB_CCDD);
        assert_eq!(
            b.calls,
            [("halfword38", 0xAABB_CCDD), ("byte29", 0xAABB_CCDD)]
        );
    }

    #[test]
    fn zero_bits_match_and_tail_word0() {
        let mut b = backend(0x0000, 0x7F);
        assert_eq!(bt_stage101_bit_relation_tail(0x1000, &mut b), 0x5566_7788);
        assert_eq!(
            b.calls,
            [
                ("halfword38", 0x1000),
                ("byte29", 0x1000),
                ("word0", 0x1000),
                ("tail", 0x1122_3344),
            ]
        );
    }

    #[test]
    fn one_bits_match_and_ignore_other_bits() {
        let mut b = backend(0xFFF0, 0xFF);
        assert_eq!(bt_stage101_bit_relation_tail(0x2000, &mut b), 0x5566_7788);
        assert_eq!(b.calls.last(), Some(&("tail", 0x1122_3344)));
    }

    #[test]
    fn only_halfword_bit4_and_byte_bit7_control_relation() {
        let mut b = backend(0xFFEF, 0x7F);
        assert_eq!(bt_stage101_bit_relation_tail(7, &mut b), 0x5566_7788);
        b.calls.clear();
        b.halfword38 = 0xFFFF;
        b.byte29 = 0x00;
        assert_eq!(bt_stage101_bit_relation_tail(7, &mut b), 7);
        assert_eq!(b.calls.len(), 2);
    }

    #[test]
    fn provenance_constants_are_current() {
        assert_eq!(STAGE101_CURRENT_BT_BIT_RELATION_TAIL_ADDR, 0x16D5A4);
        assert_eq!(STAGE101_BT_TAIL_BOUNDARY, 0x32720);
    }
}


/// Stage 102: current record formatter / candidate dispatch at `0x16D5DE`.
pub const STAGE102_CURRENT_BT_RECORD_DISPATCH_ADDR: u32 = 0x0016_D5DE;
pub const STAGE102_LEGACY_BT_RECORD_DISPATCH_ADDR: u32 = 0x0016_A862;
pub const STAGE102_BT_FIRST_BOUNDARY: u32 = 0x0002_F6C0;
pub const STAGE102_BT_SECOND_BOUNDARY: u32 = 0x0004_C440;
pub const STAGE102_BT_CANDIDATE_BOUNDARY: u32 = 0x0002_F0DC;
pub const STAGE102_BT_STAGE35_HELPER: u32 = STAGE35_CURRENT_BT_CLEAR_MASK_BIT_ADDR;
pub const STAGE102_BT_TAIL_BOUNDARY: u32 = 0x0003_2F08;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage102FirstReturn {
    /// R0 returned by `0x2F6C0`; used immediately as the record base and copied into R1.
    pub r0: u32,
    /// Caller-volatile R2 returned by `0x2F6C0`; local code leaves it live into `0x4C440`.
    pub r2: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage102Volatile23 {
    /// Caller-volatile R2 returned by `0x4C440`; forwarded unchanged into `0x2F0DC`.
    pub r2: u32,
    /// Caller-volatile R3 returned by `0x4C440`; forwarded unchanged into `0x2F0DC`.
    pub r3: u32,
}

pub trait BtStage102Backend {
    fn boundary_2f6c0(
        &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
    ) -> BtStage102FirstReturn;

    fn write_record_byte(&mut self, record: u32, offset: u32, value: u8);

    fn boundary_4c440(
        &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
    ) -> BtStage102Volatile23;

    fn boundary_2f0dc(
        &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
    ) -> u32;

    fn read_object_byte29(&mut self, object: u32) -> u8;
    fn read_candidate_byte7(&mut self, candidate: u32) -> u8;
    fn read_candidate_byte5(&mut self, candidate: u32) -> u8;

    /// Already recovered Stage-35 current helper `0x16D450`.
    ///
    /// Stage 35 proves that its observable R0 return is always the incoming
    /// passthrough R0. This callback therefore models only its side effects;
    /// Stage 102 restores the known passthrough value locally.
    fn stage35_clear_mask_bit(
        &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
    );

    fn tail_32f08(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> u32;
}

/// Exact source-level model of current `0x16D5DE`.
///
/// The low-byte bit7 of incoming R1 selects one of two record encodings. The
/// caller-volatile R2 returned by `0x2F6C0`, then R2/R3 returned by `0x4C440`,
/// are deliberately forwarded because local code does not overwrite them.
/// Candidate byte +7 is read twice; the first snapshot supplies the four-bit
/// index while the second snapshot independently gates the final tail and is
/// forwarded in R3. A Stage-35 call may occur between those two reads.
pub fn bt_stage102_record_dispatch<B: BtStage102Backend>(
    incoming_r0: u32,
    incoming_r1: u32,
    incoming_r2: u32,
    incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let object = incoming_r0;
    let packed = incoming_r1;
    let saved_r2 = incoming_r2;
    let kind = incoming_r3;

    let first = backend.boundary_2f6c0(object, packed, saved_r2, kind);
    let record = first.r0;

    let (lookup_r0, lookup_r1, second_r3) = if packed & 0x80 != 0 {
        backend.write_record_byte(record, 12, (saved_r2 | 0xFFFF_FFFE) as u8);
        let lookup_r1 = (packed >> 8) & 0xFF;
        let lookup_r0 = packed & 0x7F;
        backend.write_record_byte(record, 13, 2);
        backend.write_record_byte(record, 14, lookup_r1 as u8);
        backend.write_record_byte(record, 15, lookup_r0 as u8);
        backend.write_record_byte(record, 16, kind as u8);
        (lookup_r0, lookup_r1, 2)
    } else {
        let second_r3 = saved_r2 | 8;
        backend.write_record_byte(record, 12, second_r3 as u8);
        backend.write_record_byte(record, 13, packed as u8);
        backend.write_record_byte(record, 14, kind as u8);
        (packed, 1, second_r3)
    };

    let second = backend.boundary_4c440(object, record, first.r2, second_r3);
    let candidate =
        backend.boundary_2f0dc(lookup_r0, lookup_r1, second.r2, second.r3);
    if candidate == 0 {
        return 0;
    }

    let object_byte29 = backend.read_object_byte29(object);
    let candidate_byte7_first = backend.read_candidate_byte7(candidate);
    let relation = if saved_r2 == u32::from(object_byte29 >> 7) { 1 } else { 0 };
    let nibble = u32::from((candidate_byte7_first >> 3) & 0x0F);

    let live_r0 = if kind != 0x23 {
        let candidate_byte5 = backend.read_candidate_byte5(candidate);
        backend.stage35_clear_mask_bit(object, relation, nibble, u32::from(candidate_byte5));
        object
    } else {
        candidate
    };

    let candidate_byte7_second = backend.read_candidate_byte7(candidate);
    if candidate_byte7_second & 0x78 == 0 {
        return live_r0;
    }

    backend.tail_32f08(
        object,
        relation,
        nibble,
        u32::from(candidate_byte7_second),
    )
}

#[cfg(test)]
mod stage102_tests {
    extern crate std;
    use super::*;
    use std::vec;
    use std::vec::Vec;

    struct B {
        first: BtStage102FirstReturn,
        second: BtStage102Volatile23,
        candidate: u32,
        object_byte29: u8,
        candidate_byte5: u8,
        candidate_byte7_reads: Vec<u8>,
        mutate_second_byte7_to: Option<u8>,
        tail_return: u32,
        events: Vec<(&'static str, u32, u32, u32, u32)>,
    }

    impl Default for B {
        fn default() -> Self {
            Self {
                first: BtStage102FirstReturn { r0: 0x1000, r2: 0x2222 },
                second: BtStage102Volatile23 { r2: 0x3333, r3: 0x4444 },
                candidate: 0x2000,
                object_byte29: 0,
                candidate_byte5: 0x55,
                candidate_byte7_reads: vec![0, 0],
                mutate_second_byte7_to: None,
                tail_return: 0xDEAD_BEEF,
                events: Vec::new(),
            }
        }
    }

    impl BtStage102Backend for B {
        fn boundary_2f6c0(
            &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
        ) -> BtStage102FirstReturn {
            self.events.push(("2f6c0", r0, r1, r2, r3));
            self.first
        }

        fn write_record_byte(&mut self, record: u32, offset: u32, value: u8) {
            self.events.push(("write", record, offset, u32::from(value), 0));
        }

        fn boundary_4c440(
            &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
        ) -> BtStage102Volatile23 {
            self.events.push(("4c440", r0, r1, r2, r3));
            self.second
        }

        fn boundary_2f0dc(
            &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
        ) -> u32 {
            self.events.push(("2f0dc", r0, r1, r2, r3));
            self.candidate
        }

        fn read_object_byte29(&mut self, object: u32) -> u8 {
            self.events.push(("object29", object, u32::from(self.object_byte29), 0, 0));
            self.object_byte29
        }

        fn read_candidate_byte7(&mut self, candidate: u32) -> u8 {
            let value = self.candidate_byte7_reads.remove(0);
            self.events.push(("candidate7", candidate, u32::from(value), 0, 0));
            value
        }

        fn read_candidate_byte5(&mut self, candidate: u32) -> u8 {
            self.events.push(("candidate5", candidate, u32::from(self.candidate_byte5), 0, 0));
            self.candidate_byte5
        }

        fn stage35_clear_mask_bit(
            &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
        ) {
            self.events.push(("stage35", r0, r1, r2, r3));
            if let Some(value) = self.mutate_second_byte7_to {
                if let Some(next) = self.candidate_byte7_reads.first_mut() {
                    *next = value;
                }
            }
        }

        fn tail_32f08(
            &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
        ) -> u32 {
            self.events.push(("tail", r0, r1, r2, r3));
            self.tail_return
        }
    }

    #[test]
    fn nonnegative_format_preserves_volatile_chain_and_null_short_circuit() {
        let mut b = B { candidate: 0, ..Default::default() };
        assert_eq!(bt_stage102_record_dispatch(0xA0, 0x7F, 0, 1, &mut b), 0);
        assert_eq!(
            b.events,
            [
                ("2f6c0", 0xA0, 0x7F, 0, 1),
                ("write", 0x1000, 12, 8, 0),
                ("write", 0x1000, 13, 0x7F, 0),
                ("write", 0x1000, 14, 1, 0),
                ("4c440", 0xA0, 0x1000, 0x2222, 8),
                ("2f0dc", 0x7F, 1, 0x3333, 0x4444),
            ]
        );
    }

    #[test]
    fn negative_format_uses_high_byte_low7_and_extra_record_bytes() {
        let mut b = B {
            object_byte29: 0x80,
            candidate_byte7_reads: vec![0, 0],
            ..Default::default()
        };
        assert_eq!(
            bt_stage102_record_dispatch(0xA1, 0x1280, 1, 0x23, &mut b),
            0x2000
        );
        assert!(b.events.contains(&("write", 0x1000, 12, 0xFF, 0)));
        assert!(b.events.contains(&("write", 0x1000, 13, 2, 0)));
        assert!(b.events.contains(&("write", 0x1000, 14, 0x12, 0)));
        assert!(b.events.contains(&("write", 0x1000, 15, 0, 0)));
        assert!(b.events.contains(&("write", 0x1000, 16, 0x23, 0)));
        assert!(b.events.contains(&("4c440", 0xA1, 0x1000, 0x2222, 2)));
        assert!(b.events.contains(&("2f0dc", 0, 0x12, 0x3333, 0x4444)));
        assert!(!b.events.iter().any(|e| e.0 == "stage35"));
    }

    #[test]
    fn stage35_call_sits_between_independent_byte7_reads() {
        let mut b = B {
            object_byte29: 0x80,
            candidate_byte7_reads: vec![0x78, 0x78],
            mutate_second_byte7_to: Some(0),
            ..Default::default()
        };
        assert_eq!(
            bt_stage102_record_dispatch(0xA2, 0xAB80, 1, 0x22, &mut b),
            0xA2
        );
        let names: Vec<&str> = b.events.iter().map(|e| e.0).collect();
        let first7 = names.iter().position(|n| *n == "candidate7").unwrap();
        let helper = names.iter().position(|n| *n == "stage35").unwrap();
        let second7 = names.iter().rposition(|n| *n == "candidate7").unwrap();
        assert!(first7 < helper && helper < second7);
        assert!(b.events.contains(&("stage35", 0xA2, 1, 15, 0x55)));
    }

    #[test]
    fn tail_uses_first_snapshot_nibble_but_second_snapshot_full_byte() {
        let mut b = B {
            object_byte29: 0,
            candidate_byte7_reads: vec![0x20, 0x20],
            mutate_second_byte7_to: Some(0x78),
            tail_return: 0xCAFE,
            ..Default::default()
        };
        assert_eq!(
            bt_stage102_record_dispatch(0xA3, 0x1234, 0, 0x22, &mut b),
            0xCAFE
        );
        assert!(b.events.contains(&("stage35", 0xA3, 1, 4, 0x55)));
        assert_eq!(b.events.last(), Some(&("tail", 0xA3, 1, 4, 0x78)));
    }

    #[test]
    fn relation_compares_full_saved_r2_against_single_bit_value() {
        let mut b = B {
            object_byte29: 0x80,
            candidate_byte7_reads: vec![0x38, 0x38],
            tail_return: 0xBEEF,
            ..Default::default()
        };
        assert_eq!(
            bt_stage102_record_dispatch(0xA4, 1, 0x101, 0x23, &mut b),
            0xBEEF
        );
        assert_eq!(b.events.last(), Some(&("tail", 0xA4, 0, 7, 0x38)));
    }

    #[test]
    fn provenance_constants_are_exact() {
        assert_eq!(STAGE102_CURRENT_BT_RECORD_DISPATCH_ADDR, 0x16D5DE);
        assert_eq!(STAGE102_LEGACY_BT_RECORD_DISPATCH_ADDR, 0x16A862);
        assert_eq!(STAGE102_BT_FIRST_BOUNDARY, 0x2F6C0);
        assert_eq!(STAGE102_BT_SECOND_BOUNDARY, 0x4C440);
        assert_eq!(STAGE102_BT_CANDIDATE_BOUNDARY, 0x2F0DC);
        assert_eq!(STAGE102_BT_STAGE35_HELPER, 0x16D450);
        assert_eq!(STAGE102_BT_TAIL_BOUNDARY, 0x32F08);
    }
}

pub const STAGE103_CURRENT_BT_DISPATCH_ADDR: u32 = 0x0016_D678;
pub const STAGE103_BT_FIRST_BOUNDARY: u32 = 0x0003_35AC;
pub const STAGE103_BT_PRIMARY_LOOKUP_BOUNDARY: u32 = 0x0005_F760;
pub const STAGE103_BT_FALLBACK_LOOKUP_BOUNDARY: u32 = 0x0002_F564;
pub const STAGE103_BT_OBJECT_WORD_BOUNDARY: u32 = 0x0003_2720;
pub const STAGE103_BT_STATE_BOUNDARY: u32 = 0x0002_F5F4;
pub const STAGE103_BT_PAYLOAD_BOUNDARY: u32 = 0x0003_2CBC;
pub const STAGE103_BT_STATUS_BOUNDARY: u32 = 0x0002_F756;
pub const STAGE103_BT_FINAL_BOUNDARY: u32 = 0x000A_F248;
pub const STAGE103_BT_STACK_GUARD_FAIL: u32 = 0x0000_94C0;
pub const STAGE103_BT_GUARD_WORD_ADDR: u32 = 0x0020_0890;
pub const STAGE103_BT_CONTEXT_PTR_ADDR: u32 = 0x0020_806C;
pub const STAGE103_BT_GLOBAL_CALLBACK_ADDR: u32 = 0x0020_8170;
pub const STAGE103_BT_STATUS21_GATE_ADDR: u32 = 0x0020_81A4;
pub const STAGE103_BT_PAYLOAD_TAG: u32 = 0x0200_6DF9;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage103Regs {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
}

pub trait BtStage103Backend {
    fn read_guard_word(&mut self) -> u32;
    fn read_context_ptr(&mut self) -> u32;
    fn read_context_byte(&mut self, context: u32, offset: u32) -> u8;
    fn read_input_halfword2(&mut self, input: u32) -> u16;

    fn read_object_byte(&mut self, object: u32, offset: u32) -> u8;
    fn write_object_byte(&mut self, object: u32, offset: u32, value: u8);
    fn read_object_halfword(&mut self, object: u32, offset: u32) -> u16;
    fn write_object_halfword(&mut self, object: u32, offset: u32, value: u16);
    fn read_object_word0(&mut self, object: u32) -> u32;

    fn read_candidate_byte(&mut self, candidate: u32, offset: u32) -> u8;
    fn read_candidate_word0(&mut self, candidate: u32) -> u32;
    fn read_global_callback(&mut self) -> u32;
    fn read_status21_gate(&mut self) -> u8;

    fn boundary_335ac(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage103Regs;
    fn boundary_5f760(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage103Regs;
    fn boundary_2f564(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage103Regs;
    fn boundary_32720(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage103Regs;
    fn boundary_2f5f4(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage103Regs;
    fn indirect_call(&mut self, target: u32, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage103Regs;
    fn boundary_32cbc(&mut self, object: u32, payload: [u8; 8], r2: u32, r3: u32) -> BtStage103Regs;
    fn boundary_2f756(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage103Regs;
    fn boundary_af248(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage103Regs;
    fn boundary_94c0(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage103Regs;
}

fn stage103_finish<B: BtStage103Backend>(
    saved_guard: u32,
    mut regs: BtStage103Regs,
    backend: &mut B,
) -> u32 {
    let live_guard = backend.read_guard_word();
    if saved_guard != live_guard {
        regs = backend.boundary_94c0(regs.r0, regs.r1, saved_guard, live_guard);
    }
    regs.r0
}

fn stage103_final_boundary<B: BtStage103Backend>(
    object: u32,
    saved_guard: u32,
    backend: &mut B,
) -> u32 {
    let context = backend.read_context_ptr();
    let regs = backend.boundary_af248(context, object, 0, 0x8F);
    stage103_finish(saved_guard, regs, backend)
}

fn stage103_status_path<B: BtStage103Backend>(
    object: u32,
    mut selector: u32,
    status: u32,
    saved_guard: u32,
    backend: &mut B,
) -> u32 {
    if selector == 0x7F {
        let context = backend.read_context_ptr();
        selector = u32::from(backend.read_context_byte(context, 0x0D)) | 0x7F80;
    }

    let context = backend.read_context_ptr();
    let bit0 = u32::from(backend.read_context_byte(context, 0x0C) & 1);
    let _ = backend.boundary_2f756(object, selector, bit0, status);
    stage103_final_boundary(object, saved_guard, backend)
}

fn stage103_payload_path<B: BtStage103Backend>(
    object: u32,
    incoming_r1_high: u8,
    saved_guard: u32,
    backend: &mut B,
) -> u32 {
    let context = backend.read_context_ptr();
    let c = context.to_le_bytes();
    let payload = [0, 0, 5, incoming_r1_high, c[0], c[1], c[2], c[3]];
    let regs = backend.boundary_32cbc(object, payload, STAGE103_BT_PAYLOAD_TAG, 0);
    stage103_finish(saved_guard, regs, backend)
}

fn stage103_indirect_path<B: BtStage103Backend>(
    object: u32,
    candidate: u32,
    saved_guard: u32,
    mut regs: BtStage103Regs,
    backend: &mut B,
) -> u32 {
    let global = backend.read_global_callback();
    if global != 0 {
        regs = backend.indirect_call(global, object, regs.r1, regs.r2, global);
        if regs.r0 == 0 {
            let fallback = backend.read_candidate_word0(candidate);
            regs = backend.indirect_call(fallback, object, regs.r1, regs.r2, fallback);
        }
    } else {
        let fallback = backend.read_candidate_word0(candidate);
        regs = backend.indirect_call(fallback, object, regs.r1, regs.r2, fallback);
    }
    let _ = regs;
    stage103_final_boundary(object, saved_guard, backend)
}

/// Exact current-only source-level model of `0x16D678..0x16D7E2`.
///
/// Opaque boundaries expose full caller-volatile R0-R3 state because current
/// code forwards live outputs on several paths. Context/object/candidate bytes
/// are intentionally reread at the same path points as the firmware. The
/// payload boundary preserves the untouched high byte of pushed incoming R1.
pub fn bt_stage103_current_dispatch<B: BtStage103Backend>(
    incoming_r0: u32,
    incoming_r1: u32,
    incoming_r2: u32,
    _incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let saved_guard = backend.read_guard_word();
    let incoming_r1_high = (incoming_r1 >> 24) as u8;
    let context = backend.read_context_ptr();
    let selector_snapshot = backend.read_context_byte(context, 0x0C);
    let first_r0 = u32::from(backend.read_input_halfword2(incoming_r0));
    let first = backend.boundary_335ac(first_r0, incoming_r1, incoming_r2, context);
    let selector = u32::from(selector_snapshot >> 1);
    let object = first.r0;

    if object == 0 {
        return stage103_finish(saved_guard, first, backend);
    }

    let context = backend.read_context_ptr();
    let lookup_r0 = context.wrapping_add(0x0C);
    let lookup = if selector == 0 {
        let object_30 = backend.read_object_halfword(object, 0x30);
        let context_0d = backend.read_context_byte(context, 0x0D);
        if object_30 == 0x0F || context_0d == 4 {
            backend.boundary_5f760(
                lookup_r0,
                first.r1,
                u32::from(object_30),
                u32::from(context_0d),
            )
        } else {
            backend.boundary_2f564(
                lookup_r0,
                first.r1,
                u32::from(object_30),
                u32::from(context_0d),
            )
        }
    } else {
        backend.boundary_2f564(lookup_r0, first.r1, first.r2, context)
    };

    let candidate = lookup.r0;
    if candidate == 0 {
        return stage103_status_path(object, selector, 0x19, saved_guard, backend);
    }

    let mut regs = lookup;
    let object_f7 = backend.read_object_byte(object, 0xF7);
    if object_f7 & 8 != 0 {
        let candidate_7 = backend.read_candidate_byte(candidate, 7);
        if candidate_7 & 1 != 0 {
            let object_word0 = backend.read_object_word0(object);
            regs = backend.boundary_32720(
                object_word0,
                regs.r1,
                regs.r2,
                u32::from(candidate_7),
            );
            let post = backend.read_object_byte(object, 0xF7);
            let cleared = post & !8;
            backend.write_object_byte(object, 0xF7, cleared);
            regs.r3 = u32::from(cleared);
        } else {
            regs.r3 = u32::from(candidate_7);
        }
    } else {
        regs.r3 = u32::from(object_f7) << 28;
    }

    regs.r0 = object;
    regs.r1 = candidate;
    regs = backend.boundary_2f5f4(regs.r0, regs.r1, regs.r2, regs.r3);
    let status = regs.r0;

    if status == 0 {
        let object_0e = backend.read_object_byte(object, 0x0E);
        if object_0e & 0xFB == 3 {
            let candidate_7 = backend.read_candidate_byte(candidate, 7);
            let mask = u32::from(candidate_7 & 0x78);
            regs.r2 = mask;
            regs.r3 = u32::from(candidate_7);
            if mask == 0x20 {
                let bit = 1u16 << ((candidate_7 >> 3) & 0x0F);
                let value = backend.read_object_halfword(object, 0x0C) & !bit;
                backend.write_object_halfword(object, 0x0C, value);
                let object_1d = backend.read_object_byte(object, 0x1D);
                let next_status = if object_1d & 0x80 != 0 { 0x1F } else { 0x2A };
                return stage103_status_path(
                    object,
                    selector,
                    next_status,
                    saved_guard,
                    backend,
                );
            }
        }
        return stage103_indirect_path(object, candidate, saved_guard, regs, backend);
    }

    if status != 0x23 && status != 0x2A {
        return stage103_status_path(object, selector, status, saved_guard, backend);
    }

    if backend.read_object_byte(object, 0x1D) & 0x80 == 0 {
        return stage103_status_path(object, selector, status, saved_guard, backend);
    }

    if status != 0x2A {
        return stage103_payload_path(
            object,
            incoming_r1_high,
            saved_guard,
            backend,
        );
    }

    let candidate_7 = backend.read_candidate_byte(candidate, 7);
    let mask = candidate_7 & 0x78;
    if mask == 0x20 {
        if backend.read_object_byte(object, 0x0E) == 1 {
            let bit = 1u16 << ((candidate_7 >> 3) & 0x0F);
            let value = backend.read_object_halfword(object, 0x0C) & !bit;
            backend.write_object_halfword(object, 0x0C, value);
            return stage103_status_path(object, selector, 0x1F, saved_guard, backend);
        }
        return stage103_payload_path(object, incoming_r1_high, saved_guard, backend);
    }

    if mask == 0x08 {
        let object_0e = backend.read_object_byte(object, 0x0E);
        if object_0e == 0 {
            if backend.read_status21_gate() != 0 {
                return stage103_status_path(object, selector, 0x21, saved_guard, backend);
            }
            return stage103_payload_path(object, incoming_r1_high, saved_guard, backend);
        }
        if object_0e == 2 {
            return stage103_status_path(object, selector, 0x2A, saved_guard, backend);
        }
    }

    stage103_payload_path(object, incoming_r1_high, saved_guard, backend)
}

#[cfg(test)]
mod stage103_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum E {
        Call(&'static str, u32,u32,u32,u32),
        Indirect(u32,u32,u32,u32,u32),
        Payload(u32,[u8;8],u32,u32),
        Read(&'static str,u32,u32),
        Write(&'static str,u32,u32,u32),
    }

    struct B {
        guards: VecDeque<u32>,
        contexts: VecDeque<u32>,
        context12: VecDeque<u8>,
        context13: VecDeque<u8>,
        input_half: u16,
        object: u32,
        obj_f7: VecDeque<u8>,
        obj_0e: VecDeque<u8>,
        obj_1d: VecDeque<u8>,
        obj_30: VecDeque<u16>,
        obj_0c: u16,
        obj_word0: u32,
        cand7: VecDeque<u8>,
        cand_word0: u32,
        global_cb: u32,
        status21: u8,
        returns_335ac: VecDeque<BtStage103Regs>,
        returns_5f760: VecDeque<BtStage103Regs>,
        returns_2f564: VecDeque<BtStage103Regs>,
        returns_32720: VecDeque<BtStage103Regs>,
        returns_2f5f4: VecDeque<BtStage103Regs>,
        returns_indirect: VecDeque<BtStage103Regs>,
        returns_32cbc: VecDeque<BtStage103Regs>,
        returns_2f756: VecDeque<BtStage103Regs>,
        returns_af248: VecDeque<BtStage103Regs>,
        returns_94c0: VecDeque<BtStage103Regs>,
        e: Vec<E>,
    }

    impl Default for B {
        fn default() -> Self {
            let object=0x1000;
            Self {
                guards: VecDeque::from([0xAAAA,0xAAAA]),
                contexts: VecDeque::from([0x2000,0x2000,0x2000,0x2000,0x2000,0x2000]),
                context12: VecDeque::from([2,2,2,2,2]),
                context13: VecDeque::from([0,0,0]),
                input_half: 0x55,
                object,
                obj_f7: VecDeque::from([0]),
                obj_0e: VecDeque::from([0]),
                obj_1d: VecDeque::from([0]),
                obj_30: VecDeque::from([0]),
                obj_0c: 0xFFFF,
                obj_word0: 0x3333,
                cand7: VecDeque::from([0]),
                cand_word0: 0x8888,
                global_cb: 0,
                status21:0,
                returns_335ac: VecDeque::from([BtStage103Regs{r0:object,r1:0x11,r2:0x22,r3:0x33}]),
                returns_5f760: VecDeque::from([BtStage103Regs{r0:0x3000,r1:0x41,r2:0x42,r3:0x43}]),
                returns_2f564: VecDeque::from([BtStage103Regs{r0:0x3000,r1:0x51,r2:0x52,r3:0x53}]),
                returns_32720: VecDeque::from([BtStage103Regs{r0:9,r1:0x61,r2:0x62,r3:0x63}]),
                returns_2f5f4: VecDeque::from([BtStage103Regs{r0:1,r1:0x71,r2:0x72,r3:0x73}]),
                returns_indirect: VecDeque::new(),
                returns_32cbc: VecDeque::from([BtStage103Regs{r0:0xABCD,r1:0x81,r2:0x82,r3:0x83}]),
                returns_2f756: VecDeque::from([BtStage103Regs::default()]),
                returns_af248: VecDeque::from([BtStage103Regs{r0:0xF00D,r1:0x91,r2:0x92,r3:0x93}]),
                returns_94c0: VecDeque::from([BtStage103Regs{r0:0xBAD0,r1:0,r2:0,r3:0}]),
                e:Vec::new(),
            }
        }
    }

    impl B { fn pop(q:&mut VecDeque<BtStage103Regs>)->BtStage103Regs { q.pop_front().unwrap() } }

    impl BtStage103Backend for B {
        fn read_guard_word(&mut self)->u32 { self.guards.pop_front().unwrap() }
        fn read_context_ptr(&mut self)->u32 { let v=self.contexts.pop_front().unwrap(); self.e.push(E::Read("ctx",v,0)); v }
        fn read_context_byte(&mut self,c:u32,o:u32)->u8 { let v=if o==0x0c{self.context12.pop_front().unwrap()}else{self.context13.pop_front().unwrap()}; self.e.push(E::Read(if o==0x0c{"c12"}else{"c13"},c,u32::from(v)));v }
        fn read_input_halfword2(&mut self,i:u32)->u16{self.e.push(E::Read("in2",i,u32::from(self.input_half)));self.input_half}
        fn read_object_byte(&mut self,o:u32,off:u32)->u8{let v=match off{0xf7=>self.obj_f7.pop_front().unwrap(),0x0e=>self.obj_0e.pop_front().unwrap(),0x1d=>self.obj_1d.pop_front().unwrap(),_=>panic!()};self.e.push(E::Read("ob",o.wrapping_add(off),u32::from(v)));v}
        fn write_object_byte(&mut self,o:u32,off:u32,v:u8){self.e.push(E::Write("ob",o,off,u32::from(v)));}
        fn read_object_halfword(&mut self,o:u32,off:u32)->u16{let v=if off==0x30{self.obj_30.pop_front().unwrap()}else{self.obj_0c};self.e.push(E::Read("oh",o.wrapping_add(off),u32::from(v)));v}
        fn write_object_halfword(&mut self,o:u32,off:u32,v:u16){self.obj_0c=v;self.e.push(E::Write("oh",o,off,u32::from(v)));}
        fn read_object_word0(&mut self,o:u32)->u32{self.e.push(E::Read("ow0",o,self.obj_word0));self.obj_word0}
        fn read_candidate_byte(&mut self,c:u32,off:u32)->u8{assert_eq!(off,7);let v=self.cand7.pop_front().unwrap();self.e.push(E::Read("cb7",c,u32::from(v)));v}
        fn read_candidate_word0(&mut self,c:u32)->u32{self.e.push(E::Read("cw0",c,self.cand_word0));self.cand_word0}
        fn read_global_callback(&mut self)->u32{self.e.push(E::Read("gcb",STAGE103_BT_GLOBAL_CALLBACK_ADDR,self.global_cb));self.global_cb}
        fn read_status21_gate(&mut self)->u8{self.e.push(E::Read("s21",STAGE103_BT_STATUS21_GATE_ADDR,u32::from(self.status21)));self.status21}
        fn boundary_335ac(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage103Regs{self.e.push(E::Call("335ac",a,b,c,d));Self::pop(&mut self.returns_335ac)}
        fn boundary_5f760(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage103Regs{self.e.push(E::Call("5f760",a,b,c,d));Self::pop(&mut self.returns_5f760)}
        fn boundary_2f564(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage103Regs{self.e.push(E::Call("2f564",a,b,c,d));Self::pop(&mut self.returns_2f564)}
        fn boundary_32720(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage103Regs{self.e.push(E::Call("32720",a,b,c,d));Self::pop(&mut self.returns_32720)}
        fn boundary_2f5f4(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage103Regs{self.e.push(E::Call("2f5f4",a,b,c,d));Self::pop(&mut self.returns_2f5f4)}
        fn indirect_call(&mut self,t:u32,a:u32,b:u32,c:u32,d:u32)->BtStage103Regs{self.e.push(E::Indirect(t,a,b,c,d));Self::pop(&mut self.returns_indirect)}
        fn boundary_32cbc(&mut self,o:u32,p:[u8;8],r2:u32,r3:u32)->BtStage103Regs{self.e.push(E::Payload(o,p,r2,r3));Self::pop(&mut self.returns_32cbc)}
        fn boundary_2f756(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage103Regs{self.e.push(E::Call("2f756",a,b,c,d));Self::pop(&mut self.returns_2f756)}
        fn boundary_af248(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage103Regs{self.e.push(E::Call("af248",a,b,c,d));Self::pop(&mut self.returns_af248)}
        fn boundary_94c0(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage103Regs{self.e.push(E::Call("94c0",a,b,c,d));Self::pop(&mut self.returns_94c0)}
    }

    #[test]
    fn zero_first_boundary_exits_with_live_r1_and_guard_shape(){
        let mut b=B::default();
        b.returns_335ac=VecDeque::from([BtStage103Regs{r0:0,r1:0x1234,r2:7,r3:8}]);
        assert_eq!(bt_stage103_current_dispatch(0x9000,0xAA,0xBB,0xCC,&mut b),0);
        assert!(b.e.contains(&E::Call("335ac",0x55,0xAA,0xBB,0x2000)));
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("2f564",..)|E::Call("5f760",..))));
    }

    #[test]
    fn zero_primary_lookup_becomes_status_19(){
        let mut b=B::default(); b.context12=VecDeque::from([0,2]); b.obj_30=VecDeque::from([0x0f]);
        b.returns_5f760=VecDeque::from([BtStage103Regs{r0:0,..Default::default()}]);
        assert_eq!(bt_stage103_current_dispatch(1,2,3,4,&mut b),0xF00D);
        assert!(b.e.contains(&E::Call("5f760",0x200c,0x11,0x0f,0)));
        assert!(b.e.contains(&E::Call("2f756",0x1000,0,0,0x19)));
    }

    #[test]
    fn post_32720_reread_preserves_mutated_bits_and_clears_only_bit3(){
        let mut b=B::default(); b.obj_f7=VecDeque::from([0x08,0xAD]); b.cand7=VecDeque::from([1]);
        b.returns_2f5f4=VecDeque::from([BtStage103Regs{r0:5,..Default::default()}]); b.obj_1d=VecDeque::from([0]);
        assert_eq!(bt_stage103_current_dispatch(1,2,3,4,&mut b),0xF00D);
        assert!(b.e.contains(&E::Write("ob",0x1000,0xf7,0xA5)));
        assert!(b.e.contains(&E::Call("2f5f4",0x1000,0x3000,0x62,0xA5)));
    }

    #[test]
    fn zero_state_special_path_clears_selected_bit_and_yields_1f(){
        let mut b=B::default(); b.returns_2f5f4=VecDeque::from([BtStage103Regs{r0:0,r1:9,r2:10,r3:11}]); b.obj_0e=VecDeque::from([3]); b.cand7=VecDeque::from([0x20]); b.obj_0c=0xFFFF; b.obj_1d=VecDeque::from([0x80]);
        assert_eq!(bt_stage103_current_dispatch(1,2,3,4,&mut b),0xF00D);
        assert_eq!(b.obj_0c,0xFFEF);
        assert!(b.e.contains(&E::Call("2f756",0x1000,1,0,0x1F)));
    }

    #[test]
    fn nonzero_2a_mask20_object1_clears_selected_bit(){
        let mut b=B::default(); b.returns_2f5f4=VecDeque::from([BtStage103Regs{r0:0x2a,..Default::default()}]); b.obj_1d=VecDeque::from([0x80]); b.cand7=VecDeque::from([0x20]); b.obj_0e=VecDeque::from([1]); b.obj_0c=0xFFFF;
        assert_eq!(bt_stage103_current_dispatch(1,2,3,4,&mut b),0xF00D);
        assert_eq!(b.obj_0c,0xFFEF);
        assert!(b.e.contains(&E::Call("2f756",0x1000,1,0,0x1F)));
    }

    #[test]
    fn status21_gate_is_reread_on_exact_2a_mask8_object0_path(){
        let mut b=B::default(); b.returns_2f5f4=VecDeque::from([BtStage103Regs{r0:0x2a,..Default::default()}]); b.obj_1d=VecDeque::from([0x80]); b.cand7=VecDeque::from([0x08]); b.obj_0e=VecDeque::from([0]); b.status21=1;
        assert_eq!(bt_stage103_current_dispatch(1,2,3,4,&mut b),0xF00D);
        assert!(b.e.contains(&E::Read("s21",STAGE103_BT_STATUS21_GATE_ADDR,1)));
        assert!(b.e.contains(&E::Call("2f756",0x1000,1,0,0x21)));
    }

    #[test]
    fn payload_preserves_incoming_r1_high_byte_and_live_context(){
        let mut b=B::default(); b.returns_2f5f4=VecDeque::from([BtStage103Regs{r0:0x23,..Default::default()}]); b.obj_1d=VecDeque::from([0x80]);
        assert_eq!(bt_stage103_current_dispatch(1,0xAB00_0002,3,4,&mut b),0xABCD);
        assert!(b.e.iter().any(|e|matches!(e,E::Payload(0x1000,[0,0,5,0xAB,0,0x20,0,0],STAGE103_BT_PAYLOAD_TAG,0))));
    }

    #[test]
    fn two_stage_indirect_callback_forwards_returned_r1_r2_to_fallback(){
        let mut b=B::default(); b.returns_2f5f4=VecDeque::from([BtStage103Regs{r0:0,r1:0x1111,r2:0x2222,r3:0}]); b.obj_0e=VecDeque::from([0]); b.global_cb=0x4444; b.cand_word0=0x5555;
        b.returns_indirect=VecDeque::from([BtStage103Regs{r0:0,r1:0xAAAA,r2:0xBBBB,r3:0xCCCC},BtStage103Regs{r0:9,r1:1,r2:2,r3:3}]);
        assert_eq!(bt_stage103_current_dispatch(1,2,3,4,&mut b),0xF00D);
        assert!(b.e.contains(&E::Indirect(0x4444,0x1000,0x1111,0x2222,0x4444)));
        assert!(b.e.contains(&E::Indirect(0x5555,0x1000,0xAAAA,0xBBBB,0x5555)));
    }

    #[test]
    fn selector_7f_uses_distinct_context_rereads(){
        let mut b=B::default(); b.context12=VecDeque::from([0xFE,3]); b.context13=VecDeque::from([0x12]); b.contexts=VecDeque::from([0x2000,0x2100,0x2200,0x2300,0x2400]);
        b.returns_2f564=VecDeque::from([BtStage103Regs{r0:0,..Default::default()}]);
        assert_eq!(bt_stage103_current_dispatch(1,2,3,4,&mut b),0xF00D);
        assert!(b.e.contains(&E::Read("c13",0x2200,0x12)));
        assert!(b.e.contains(&E::Read("c12",0x2300,3)));
        assert!(b.e.contains(&E::Call("2f756",0x1000,0x7F92,1,0x19)));
        assert!(b.e.contains(&E::Call("af248",0x2400,0x1000,0,0x8f)));
    }

    #[test]
    fn guard_mismatch_can_replace_final_r0_and_receives_live_r1(){
        let mut b=B::default(); b.guards=VecDeque::from([0xAAAA,0xBBBB]); b.returns_335ac=VecDeque::from([BtStage103Regs{r0:0,r1:0xCAFE,r2:7,r3:8}]); b.returns_94c0=VecDeque::from([BtStage103Regs{r0:0xBAD0,..Default::default()}]);
        assert_eq!(bt_stage103_current_dispatch(1,2,3,4,&mut b),0xBAD0);
        assert!(b.e.contains(&E::Call("94c0",0,0xCAFE,0xAAAA,0xBBBB)));
    }
}

pub const STAGE104_CURRENT_BT_CONSTANT_RETURN_ADDR: u32 = 0x0016_D7F8;
pub const STAGE104_BT_CONSTANT_RETURN_VALUE: u32 = 12;

/// Exact source-level model of current `0x16D7F8..0x16D7FC`.
///
/// The four-byte leaf is `MOVS R0,#12; BX LR`: it performs no memory access,
/// no calls, and no conditional control flow. Incoming register values and
/// ambient state therefore cannot affect the returned value.
pub fn bt_stage104_constant_return(
    _incoming_r0: u32,
    _incoming_r1: u32,
    _incoming_r2: u32,
    _incoming_r3: u32,
) -> u32 {
    STAGE104_BT_CONSTANT_RETURN_VALUE
}

#[cfg(test)]
mod stage104_tests {
    use super::*;

    #[test]
    fn always_returns_exact_literal_twelve() {
        for inputs in [
            [0, 0, 0, 0],
            [1, 2, 3, 4],
            [u32::MAX, 0x8000_0000, 0x1234_5678, 0xA5A5_5A5A],
        ] {
            assert_eq!(
                bt_stage104_constant_return(inputs[0], inputs[1], inputs[2], inputs[3]),
                12
            );
        }
    }

    #[test]
    fn provenance_constants_are_exact() {
        assert_eq!(STAGE104_CURRENT_BT_CONSTANT_RETURN_ADDR, 0x16D7F8);
        assert_eq!(STAGE104_BT_CONSTANT_RETURN_VALUE, 12);
    }
}

pub const STAGE105_CURRENT_BT_STATE_DISPATCH_ADDR: u32 = 0x0016_D7FC;
pub const STAGE105_LEGACY_BT_STATE_DISPATCH_ADDR: u32 = 0x0016_A900;
pub const STAGE105_BT_STATE_BASE: u32 = 0x0020_AEC0;
pub const STAGE105_BT_GATE_ADDR: u32 = 0x0020_2D64;
pub const STAGE105_BT_OPTIONAL_WORD_ADDR: u32 = 0x0020_AECC;
pub const STAGE105_BT_COMMON_BASE_ADDR: u32 = 0x0020_9BB4;
pub const STAGE105_BT_MODE_FLAG_ADDR: u32 = 0x0020_2BEA;
pub const STAGE105_BT_PUBLISH_WORD_ADDR: u32 = 0x0020_AE7C;
pub const STAGE105_BT_FLAG_BASE_ADDR: u32 = 0x0020_8338;
pub const STAGE105_BT_SOURCE_PTR_ADDR: u32 = 0x0022_1ED8;
pub const STAGE105_LEGACY_BT_SOURCE_PTR_ADDR: u32 = 0x0022_1EA4;
pub const STAGE105_BT_DEST_BYTE_ADDR: u32 = 0x0020_AE70;

pub const STAGE105_BT_FIRST_BOUNDARY: u32 = 0x0003_5060;
pub const STAGE105_BT_EARLY_TAIL: u32 = 0x0006_28D0;
pub const STAGE105_BT_PREP_BOUNDARY: u32 = 0x0006_2550;
pub const STAGE105_BT_ENABLE_BOUNDARY: u32 = 0x0006_2510;
pub const STAGE105_BT_SIGNED_BOUNDARY: u32 = 0x0003_0FF4;
pub const STAGE105_BT_OBJECT_BOUNDARY: u32 = 0x0003_3B50;
pub const STAGE105_BT_BUFFER_BOUNDARY: u32 = 0x0000_3D24;
pub const STAGE105_BT_MODE_BOUNDARY: u32 = 0x0006_274E;
pub const STAGE105_BT_MODE_APPLY_BOUNDARY: u32 = 0x0006_2710;
pub const STAGE105_BT_SECOND_APPLY_BOUNDARY: u32 = 0x0003_5760;
pub const STAGE105_BT_PUBLISH_BOUNDARY: u32 = 0x000F_F594;
pub const STAGE105_BT_FINAL_EXTRA_BOUNDARY: u32 = 0x0006_2204;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage105Regs {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
}

pub trait BtStage105Backend {
    fn read_input_word(&mut self, input_ptr: u32) -> u32;
    fn read_state_byte(&mut self, offset: u32) -> u8;
    fn write_state_byte(&mut self, offset: u32, value: u8);
    fn read_gate_byte(&mut self) -> u8;
    fn read_optional_word(&mut self) -> u32;
    fn write_mode_flag(&mut self, value: u8);
    fn write_publish_word(&mut self, value: u32);
    fn read_flag_byte13(&mut self) -> u8;
    fn write_flag_byte13(&mut self, value: u8);
    fn read_source_ptr(&mut self) -> u32;
    fn read_indirect_byte(&mut self, ptr: u32) -> u8;
    fn write_indirect_byte(&mut self, ptr: u32, value: u8);
    fn write_dest_byte(&mut self, value: u8);

    fn boundary_35060(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn tail_628d0(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_62550(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_62510(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_30ff4(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_33b50(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_3d24(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_6274e(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_62710(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_35760(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_ff594(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
    fn boundary_62204(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage105Regs;
}

/// Exact source-level model of current `0x16D7FC..0x16D8E8`.
///
/// The model preserves caller-volatile R0-R3 across opaque boundaries whenever
/// the machine code forwards them, including the early-tail R2 plus restored
/// incoming R3 shape, independent state-byte rereads, the SXT.B truncation, and
/// the final path-dependent return source.
pub fn bt_stage105_state_dispatch<B: BtStage105Backend>(
    incoming_r0: u32,
    incoming_r1: u32,
    incoming_r2: u32,
    incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let first_input_word = backend.read_input_word(incoming_r0);
    let byte7_snapshot = backend.read_state_byte(7);
    let mut regs = backend.boundary_35060(first_input_word, incoming_r1, incoming_r2, incoming_r3);
    let saved_first_r0 = regs.r0;

    let byte1 = backend.read_state_byte(1);
    let high_nibble = byte7_snapshot & 0xF0;
    let low_nibble = byte7_snapshot & 0x0F;

    if byte1 == 0xFF {
        let fresh_input_word = backend.read_input_word(incoming_r0);
        let tail = backend.tail_628d0(fresh_input_word, 0, regs.r2, incoming_r3);
        return tail.r0;
    }

    regs.r0 = u32::from(byte1);
    regs = backend.boundary_62550(regs.r0, regs.r1, regs.r2, regs.r3);
    regs.r0 = 1;
    regs = backend.boundary_62510(regs.r0, regs.r1, regs.r2, regs.r3);

    let byte7_reread = backend.read_state_byte(7);
    backend.write_state_byte(7, byte7_reread & 0x0F);

    let gate = backend.read_gate_byte();
    regs.r3 = STAGE105_BT_GATE_ADDR;
    if gate == 0 {
        let byte5 = backend.read_state_byte(5);
        let scaled = byte5.wrapping_mul(252);
        regs.r0 = (scaled as i8 as i32) as u32;
        regs = backend.boundary_30ff4(regs.r0, regs.r1, regs.r2, regs.r3);
        regs.r1 = regs.r0;
        regs.r0 = incoming_r0;
        regs = backend.boundary_33b50(regs.r0, regs.r1, regs.r2, regs.r3);

        regs.r3 = STAGE105_BT_OPTIONAL_WORD_ADDR;
        let optional_word = backend.read_optional_word();
        regs.r0 = optional_word;
        if optional_word != 0 {
            regs.r2 = 0x1C;
            regs.r1 = u32::from(gate);
            regs = backend.boundary_3d24(regs.r0, regs.r1, regs.r2, regs.r3);
        }
    }

    regs.r2 = 0x40;
    regs.r1 = 0;
    regs.r0 = STAGE105_BT_COMMON_BASE_ADDR;
    regs = backend.boundary_3d24(regs.r0, regs.r1, regs.r2, regs.r3);

    regs.r3 = STAGE105_BT_MODE_FLAG_ADDR;
    regs.r0 = 1;
    backend.write_mode_flag(1);

    let mode = if low_nibble == 7 {
        if high_nibble == 0 {
            5
        } else if high_nibble == 0x10 {
            4
        } else if high_nibble == 0x30 {
            3
        } else {
            0
        }
    } else {
        regs.r1 = u32::from(low_nibble);
        if high_nibble <= 0x10 {
            regs.r0 = 0;
        }
        regs = backend.boundary_6274e(regs.r0, regs.r1, regs.r2, regs.r3);
        regs.r0
    };

    regs.r1 = mode;
    regs.r0 = saved_first_r0;
    regs = backend.boundary_62710(regs.r0, regs.r1, regs.r2, regs.r3);

    regs.r1 = mode;
    regs.r0 = saved_first_r0;
    regs = backend.boundary_35760(regs.r0, regs.r1, regs.r2, regs.r3);

    if mode > 2 {
        regs.r1 = u32::from(low_nibble);
        regs.r0 = mode;
        regs = backend.boundary_ff594(regs.r0, regs.r1, regs.r2, regs.r3);
        backend.write_publish_word(regs.r0);
        regs.r3 = STAGE105_BT_PUBLISH_WORD_ADDR;
        regs.r0 = saved_first_r0;
        regs = backend.boundary_62204(regs.r0, regs.r1, regs.r2, regs.r3);

        let flag = backend.read_flag_byte13();
        backend.write_flag_byte13(flag | 0x10);
    }

    let source_ptr = backend.read_source_ptr();
    let source_byte = backend.read_indirect_byte(source_ptr);
    backend.write_dest_byte(source_byte);
    backend.write_indirect_byte(source_ptr, 0);
    regs.r0
}

#[cfg(test)]
mod stage105_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum E {
        Call(&'static str, u32, u32, u32, u32),
        Read(&'static str, u32, u32),
        Write(&'static str, u32, u32),
    }

    struct B {
        input_words: VecDeque<u32>,
        state1: u8,
        state5: u8,
        state7: VecDeque<u8>,
        gate: u8,
        optional_word: u32,
        flag13: u8,
        source_ptr: u32,
        source_byte: u8,
        returns_35060: VecDeque<BtStage105Regs>,
        returns_628d0: VecDeque<BtStage105Regs>,
        returns_62550: VecDeque<BtStage105Regs>,
        returns_62510: VecDeque<BtStage105Regs>,
        returns_30ff4: VecDeque<BtStage105Regs>,
        returns_33b50: VecDeque<BtStage105Regs>,
        returns_3d24: VecDeque<BtStage105Regs>,
        returns_6274e: VecDeque<BtStage105Regs>,
        returns_62710: VecDeque<BtStage105Regs>,
        returns_35760: VecDeque<BtStage105Regs>,
        returns_ff594: VecDeque<BtStage105Regs>,
        returns_62204: VecDeque<BtStage105Regs>,
        e: Vec<E>,
    }

    impl Default for B {
        fn default() -> Self {
            Self {
                input_words: VecDeque::from([0x1111, 0x2222]),
                state1: 1,
                state5: 0,
                state7: VecDeque::from([0x20, 0x20]),
                gate: 1,
                optional_word: 0,
                flag13: 0,
                source_ptr: 0x9000,
                source_byte: 0x5A,
                returns_35060: VecDeque::from([BtStage105Regs{r0:0x7000,r1:0x11,r2:0x22,r3:0x33}]),
                returns_628d0: VecDeque::from([BtStage105Regs{r0:0xDEAD,..Default::default()}]),
                returns_62550: VecDeque::from([BtStage105Regs{r0:0x51,r1:0x52,r2:0x53,r3:0x54}]),
                returns_62510: VecDeque::from([BtStage105Regs{r0:0x61,r1:0x62,r2:0x63,r3:0x64}]),
                returns_30ff4: VecDeque::from([BtStage105Regs{r0:0x71,r1:0x72,r2:0x73,r3:0x74}]),
                returns_33b50: VecDeque::from([BtStage105Regs{r0:0x81,r1:0x82,r2:0x83,r3:0x84}]),
                returns_3d24: VecDeque::from([BtStage105Regs{r0:0x91,r1:0x92,r2:0x93,r3:0x94}]),
                returns_6274e: VecDeque::from([BtStage105Regs{r0:1,r1:0xA2,r2:0xA3,r3:0xA4}]),
                returns_62710: VecDeque::from([BtStage105Regs{r0:0xB1,r1:0xB2,r2:0xB3,r3:0xB4}]),
                returns_35760: VecDeque::from([BtStage105Regs{r0:0xCAFE,r1:0xC2,r2:0xC3,r3:0xC4}]),
                returns_ff594: VecDeque::from([BtStage105Regs{r0:0xD1,r1:0xD2,r2:0xD3,r3:0xD4}]),
                returns_62204: VecDeque::from([BtStage105Regs{r0:0xBEEF,r1:0xE2,r2:0xE3,r3:0xE4}]),
                e: Vec::new(),
            }
        }
    }

    impl B {
        fn pop(q: &mut VecDeque<BtStage105Regs>) -> BtStage105Regs { q.pop_front().unwrap() }
    }

    impl BtStage105Backend for B {
        fn read_input_word(&mut self,p:u32)->u32{let v=self.input_words.pop_front().unwrap();self.e.push(E::Read("input",p,v));v}
        fn read_state_byte(&mut self,o:u32)->u8{let v=match o{1=>self.state1,5=>self.state5,7=>self.state7.pop_front().unwrap(),_=>panic!()};self.e.push(E::Read("state",o,u32::from(v)));v}
        fn write_state_byte(&mut self,o:u32,v:u8){self.e.push(E::Write("state",o,u32::from(v)));}
        fn read_gate_byte(&mut self)->u8{self.e.push(E::Read("gate",STAGE105_BT_GATE_ADDR,u32::from(self.gate)));self.gate}
        fn read_optional_word(&mut self)->u32{self.e.push(E::Read("optional",STAGE105_BT_OPTIONAL_WORD_ADDR,self.optional_word));self.optional_word}
        fn write_mode_flag(&mut self,v:u8){self.e.push(E::Write("modeflag",STAGE105_BT_MODE_FLAG_ADDR,u32::from(v)));}
        fn write_publish_word(&mut self,v:u32){self.e.push(E::Write("publish",STAGE105_BT_PUBLISH_WORD_ADDR,v));}
        fn read_flag_byte13(&mut self)->u8{self.e.push(E::Read("flag13",STAGE105_BT_FLAG_BASE_ADDR+0x13,u32::from(self.flag13)));self.flag13}
        fn write_flag_byte13(&mut self,v:u8){self.flag13=v;self.e.push(E::Write("flag13",STAGE105_BT_FLAG_BASE_ADDR+0x13,u32::from(v)));}
        fn read_source_ptr(&mut self)->u32{self.e.push(E::Read("srcptr",STAGE105_BT_SOURCE_PTR_ADDR,self.source_ptr));self.source_ptr}
        fn read_indirect_byte(&mut self,p:u32)->u8{self.e.push(E::Read("srcbyte",p,u32::from(self.source_byte)));self.source_byte}
        fn write_indirect_byte(&mut self,p:u32,v:u8){self.source_byte=v;self.e.push(E::Write("srcbyte",p,u32::from(v)));}
        fn write_dest_byte(&mut self,v:u8){self.e.push(E::Write("dest",STAGE105_BT_DEST_BYTE_ADDR,u32::from(v)));}
        fn boundary_35060(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("35060",a,b,c,d));Self::pop(&mut self.returns_35060)}
        fn tail_628d0(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("628d0",a,b,c,d));Self::pop(&mut self.returns_628d0)}
        fn boundary_62550(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("62550",a,b,c,d));Self::pop(&mut self.returns_62550)}
        fn boundary_62510(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("62510",a,b,c,d));Self::pop(&mut self.returns_62510)}
        fn boundary_30ff4(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("30ff4",a,b,c,d));Self::pop(&mut self.returns_30ff4)}
        fn boundary_33b50(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("33b50",a,b,c,d));Self::pop(&mut self.returns_33b50)}
        fn boundary_3d24(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("3d24",a,b,c,d));Self::pop(&mut self.returns_3d24)}
        fn boundary_6274e(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("6274e",a,b,c,d));Self::pop(&mut self.returns_6274e)}
        fn boundary_62710(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("62710",a,b,c,d));Self::pop(&mut self.returns_62710)}
        fn boundary_35760(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("35760",a,b,c,d));Self::pop(&mut self.returns_35760)}
        fn boundary_ff594(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("ff594",a,b,c,d));Self::pop(&mut self.returns_ff594)}
        fn boundary_62204(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage105Regs{self.e.push(E::Call("62204",a,b,c,d));Self::pop(&mut self.returns_62204)}
    }

    #[test]
    fn early_tail_rereads_word_forwards_live_r2_and_restores_incoming_r3(){
        let mut b=B::default(); b.state1=0xff; b.state7=VecDeque::from([0xab]);
        b.returns_35060=VecDeque::from([BtStage105Regs{r0:7,r1:8,r2:0xCAFE,r3:0xDEAD}]);
        assert_eq!(bt_stage105_state_dispatch(0x1000,2,3,0x4444,&mut b),0xDEAD);
        assert!(b.e.contains(&E::Call("35060",0x1111,2,3,0x4444)));
        assert!(b.e.contains(&E::Call("628d0",0x2222,0,0xCAFE,0x4444)));
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("62550",..))));
    }

    #[test]
    fn gate_zero_preserves_sxtb_and_30ff4_to_33b50_volatile_chain(){
        let mut b=B::default(); b.gate=0; b.state5=0x20; b.optional_word=0; b.state7=VecDeque::from([0x20,0x2f]);
        b.returns_62510=VecDeque::from([BtStage105Regs{r0:0x61,r1:0x1111,r2:0x2222,r3:0x3333}]);
        b.returns_30ff4=VecDeque::from([BtStage105Regs{r0:0x7777,r1:0xAAAA,r2:0xBBBB,r3:0xCCCC}]);
        b.returns_3d24=VecDeque::from([BtStage105Regs{r0:0x91,r1:0x92,r2:0x93,r3:0x94}]);
        assert_eq!(bt_stage105_state_dispatch(0x1000,2,3,4,&mut b),0xCAFE);
        assert!(b.e.contains(&E::Write("state",7,0x0f)));
        assert!(b.e.contains(&E::Call("30ff4",0xffff_ff80,0x1111,0x2222,STAGE105_BT_GATE_ADDR)));
        assert!(b.e.contains(&E::Call("33b50",0x1000,0x7777,0xBBBB,0xCCCC)));
        assert!(b.e.contains(&E::Call("3d24",STAGE105_BT_COMMON_BASE_ADDR,0,0x40,STAGE105_BT_OPTIONAL_WORD_ADDR)));
    }

    #[test]
    fn optional_first_buffer_call_forwards_its_returned_r3_into_common_call(){
        let mut b=B::default(); b.gate=0; b.optional_word=0x1234; b.state7=VecDeque::from([0x20,0x20]);
        b.returns_3d24=VecDeque::from([
            BtStage105Regs{r0:1,r1:2,r2:3,r3:0xFEED},
            BtStage105Regs{r0:0x91,r1:0x92,r2:0x93,r3:0x94},
        ]);
        assert_eq!(bt_stage105_state_dispatch(0x1000,2,3,4,&mut b),0xCAFE);
        assert!(b.e.contains(&E::Call("3d24",0x1234,0,0x1c,STAGE105_BT_OPTIONAL_WORD_ADDR)));
        assert!(b.e.contains(&E::Call("3d24",STAGE105_BT_COMMON_BASE_ADDR,0,0x40,0xFEED)));
    }

    #[test]
    fn nonzero_gate_skips_signed_path_and_common_call_keeps_gate_address_in_r3(){
        let mut b=B::default(); b.gate=9; b.state7=VecDeque::from([0x20,0x20]);
        assert_eq!(bt_stage105_state_dispatch(0x1000,2,3,4,&mut b),0xCAFE);
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("30ff4",..)|E::Call("33b50",..))));
        assert!(b.e.contains(&E::Call("3d24",STAGE105_BT_COMMON_BASE_ADDR,0,0x40,STAGE105_BT_GATE_ADDR)));
    }

    #[test]
    fn low7_uses_exact_literal_mode_map_without_mode_boundary(){
        for (snapshot, expected_mode) in [(0x07,5u32),(0x17,4),(0x37,3),(0x27,0)] {
            let mut b=B::default(); b.state7=VecDeque::from([snapshot,snapshot]);
            b.returns_62710=VecDeque::from([BtStage105Regs{r0:1,r1:2,r2:3,r3:4}]);
            b.returns_35760=VecDeque::from([BtStage105Regs{r0:0xCAFE,r1:2,r2:3,r3:4}]);
            if expected_mode>2 { b.returns_ff594=VecDeque::from([BtStage105Regs{r0:0xD1,r1:2,r2:3,r3:4}]); b.returns_62204=VecDeque::from([BtStage105Regs{r0:0xBEEF,r1:2,r2:3,r3:4}]); }
            let _=bt_stage105_state_dispatch(0x1000,2,3,4,&mut b);
            assert!(!b.e.iter().any(|e|matches!(e,E::Call("6274e",..))));
            assert!(b.e.iter().any(|e|matches!(e,E::Call("62710",0x7000,m,_,_) if *m==expected_mode)));
        }
    }

    #[test]
    fn non7_mode_boundary_gets_boolean_high_nibble_gate_and_live_r2_r3(){
        let mut b=B::default(); b.state7=VecDeque::from([0x25,0x25]);
        b.returns_3d24=VecDeque::from([BtStage105Regs{r0:0x91,r1:0x92,r2:0xABCD,r3:0xDCBA}]);
        b.returns_6274e=VecDeque::from([BtStage105Regs{r0:2,r1:0xA2,r2:0xA3,r3:0xA4}]);
        assert_eq!(bt_stage105_state_dispatch(0x1000,2,3,4,&mut b),0xCAFE);
        assert!(b.e.contains(&E::Call("6274e",1,5,0xABCD,STAGE105_BT_MODE_FLAG_ADDR)));
        assert!(b.e.contains(&E::Call("62710",0x7000,2,0xA3,0xA4)));
    }

    #[test]
    fn mode_at_most_two_returns_35760_and_still_moves_final_byte(){
        let mut b=B::default(); b.state7=VecDeque::from([0x25,0x25]); b.source_byte=0x6b;
        b.returns_6274e=VecDeque::from([BtStage105Regs{r0:2,r1:3,r2:4,r3:5}]);
        b.returns_35760=VecDeque::from([BtStage105Regs{r0:0xCAFE,r1:0xC2,r2:0xC3,r3:0xC4}]);
        assert_eq!(bt_stage105_state_dispatch(0x1000,2,3,4,&mut b),0xCAFE);
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("ff594",..)|E::Call("62204",..))));
        assert!(b.e.contains(&E::Write("dest",STAGE105_BT_DEST_BYTE_ADDR,0x6b)));
        assert!(b.e.contains(&E::Write("srcbyte",0x9000,0)));
    }

    #[test]
    fn mode_above_two_publishes_ff594_then_returns_62204_and_sets_bit4(){
        let mut b=B::default(); b.state7=VecDeque::from([0x37,0x37]); b.flag13=0x81; b.source_byte=0x44;
        b.returns_35760=VecDeque::from([BtStage105Regs{r0:0xC1,r1:0xC2,r2:0x1234,r3:0x5678}]);
        b.returns_ff594=VecDeque::from([BtStage105Regs{r0:0xD00D,r1:0xAAAA,r2:0xBBBB,r3:0xCCCC}]);
        b.returns_62204=VecDeque::from([BtStage105Regs{r0:0xBEEF,r1:1,r2:2,r3:3}]);
        assert_eq!(bt_stage105_state_dispatch(0x1000,2,3,4,&mut b),0xBEEF);
        assert!(b.e.contains(&E::Call("ff594",3,7,0x1234,0x5678)));
        assert!(b.e.contains(&E::Write("publish",STAGE105_BT_PUBLISH_WORD_ADDR,0xD00D)));
        assert!(b.e.contains(&E::Call("62204",0x7000,0xAAAA,0xBBBB,STAGE105_BT_PUBLISH_WORD_ADDR)));
        assert!(b.e.contains(&E::Write("flag13",STAGE105_BT_FLAG_BASE_ADDR+0x13,0x91)));
    }

    #[test]
    fn provenance_constants_are_exact(){
        assert_eq!(STAGE105_CURRENT_BT_STATE_DISPATCH_ADDR,0x16D7FC);
        assert_eq!(STAGE105_LEGACY_BT_STATE_DISPATCH_ADDR,0x16A900);
        assert_eq!(STAGE105_BT_FIRST_BOUNDARY,0x35060);
        assert_eq!(STAGE105_BT_EARLY_TAIL,0x628D0);
        assert_eq!(STAGE105_BT_SOURCE_PTR_ADDR,0x221ED8);
        assert_eq!(STAGE105_LEGACY_BT_SOURCE_PTR_ADDR,0x221EA4);
    }
}

pub const STAGE106_CURRENT_BT_OBJECT_DISPATCH_ADDR: u32 = 0x0016_D90C;
pub const STAGE106_LEGACY_BT_OBJECT_DISPATCH_ADDR: u32 = 0x0016_AA10;
pub const STAGE106_CURRENT_BODY_LEN: u32 = 142;
pub const STAGE106_BT_MASK: u32 = 0x0000_3306;

pub const STAGE106_BT_PRECHECK_BOUNDARY: u32 = 0x0003_CCDC;
pub const STAGE106_BT_FIRST_TAIL: u32 = 0x0003_CC9E;
pub const STAGE106_BT_COMMON_BOUNDARY: u32 = 0x0004_BC44;
pub const STAGE106_BT_XOR_BOUNDARY: u32 = 0x0006_F246;
pub const STAGE106_BT_FINAL_TAIL: u32 = 0x0004_14A0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage106Regs {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
}

pub trait BtStage106Backend {
    fn read_object_halfword(&mut self, object: u32, offset: u32) -> u16;
    fn write_object_halfword(&mut self, object: u32, offset: u32, value: u16);
    fn read_object_byte(&mut self, object: u32, offset: u32) -> u8;
    fn write_object_byte(&mut self, object: u32, offset: u32, value: u8);

    fn boundary_3ccdc(
        &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
    ) -> BtStage106Regs;
    fn tail_3cc9e(
        &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
    ) -> BtStage106Regs;
    fn boundary_4bc44(
        &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
    ) -> BtStage106Regs;
    fn boundary_6f246(
        &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
    ) -> BtStage106Regs;
    fn tail_414a0(
        &mut self, r0: u32, r1: u32, r2: u32, r3: u32,
    ) -> BtStage106Regs;
}

fn stage106_set_bit3<B: BtStage106Backend>(
    object: u32,
    backend: &mut B,
) {
    let value = backend.read_object_byte(object, 0x1F);
    backend.write_object_byte(object, 0x1F, value | 8);
}

fn stage106_common<B: BtStage106Backend>(
    object: u32,
    mut regs: BtStage106Regs,
    backend: &mut B,
) -> u32 {
    let copied_ec = backend.read_object_halfword(object, 0xEC);
    backend.write_object_halfword(object, 0x68, copied_ec);
    regs.r3 = u32::from(copied_ec);
    regs.r0 = object;
    regs = backend.boundary_4bc44(regs.r0, regs.r1, regs.r2, regs.r3);

    let fresh_ec = backend.read_object_halfword(object, 0xEC);
    let fresh_64 = backend.read_object_halfword(object, 0x64);
    regs.r2 = u32::from(fresh_ec) ^ STAGE106_BT_MASK;
    regs.r1 = u32::from(fresh_64);
    regs.r0 = 0;
    regs = backend.boundary_6f246(regs.r0, regs.r1, regs.r2, regs.r3);

    let tail = backend.tail_414a0(object, 1, 0, regs.r3);
    tail.r0
}

/// Exact source-level model of current `0x16D90C..0x16D99A`.
///
/// Incoming R1-R3 are overwritten before the first opaque boundary, matching
/// the machine code. The model preserves the two distinct 0x3CC9E tail shapes,
/// local returns, independent +0xEC rereads, and live caller-volatile R3 edges
/// through the common boundary chain.
pub fn bt_stage106_object_dispatch<B: BtStage106Backend>(
    object: u32,
    _incoming_r1: u32,
    _incoming_r2: u32,
    _incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let first_ec = backend.read_object_halfword(object, 0xEC);
    let byte_a7 = backend.read_object_byte(object, 0xA7);
    let masked = u32::from(first_ec) & STAGE106_BT_MASK;
    let mut regs = BtStage106Regs {
        r0: object,
        r1: u32::from(first_ec),
        r2: masked,
        r3: u32::from(byte_a7),
    };

    if masked == 0 {
        regs.r1 = u32::from(byte_a7 & 0xE0);
        if regs.r1 != 0 {
            return stage106_common(object, regs, backend);
        }

        stage106_set_bit3(object, backend);
        let byte_eb = backend.read_object_byte(object, 0xEB);
        regs.r3 = u32::from(byte_eb);
        if byte_eb & 0x30 == 0 {
            return regs.r0;
        }

        let tail = backend.tail_3cc9e(object, 0, 0, regs.r3);
        return tail.r0;
    }

    if byte_a7 & 0xE0 == 0 {
        return stage106_common(object, regs, backend);
    }

    regs = backend.boundary_3ccdc(regs.r0, regs.r1, regs.r2, regs.r3);
    if regs.r0 == 0 {
        return stage106_common(object, regs, backend);
    }

    stage106_set_bit3(object, backend);
    let byte_eb = backend.read_object_byte(object, 0xEB);
    regs.r3 = u32::from(byte_eb & 0x30);
    if regs.r3 == 0x10 {
        return regs.r0;
    }

    regs.r1 = 1;
    regs.r0 = object;
    let tail = backend.tail_3cc9e(regs.r0, regs.r1, regs.r2, regs.r3);
    tail.r0
}

#[cfg(test)]
mod stage106_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum E {
        Call(&'static str, u32, u32, u32, u32),
        Read(&'static str, u32, u32),
        Write(&'static str, u32, u32, u32),
    }

    struct B {
        ec: VecDeque<u16>,
        h64: VecDeque<u16>,
        a7: u8,
        b1f: VecDeque<u8>,
        beb: VecDeque<u8>,
        r_3ccdc: VecDeque<BtStage106Regs>,
        r_3cc9e: VecDeque<BtStage106Regs>,
        r_4bc44: VecDeque<BtStage106Regs>,
        r_6f246: VecDeque<BtStage106Regs>,
        r_414a0: VecDeque<BtStage106Regs>,
        e: Vec<E>,
    }

    impl Default for B {
        fn default() -> Self {
            Self {
                ec: VecDeque::from([0, 0x1111, 0x2222]),
                h64: VecDeque::from([0x3333]),
                a7: 0,
                b1f: VecDeque::from([0x40]),
                beb: VecDeque::from([0]),
                r_3ccdc: VecDeque::from([BtStage106Regs{r0:1,r1:2,r2:3,r3:4}]),
                r_3cc9e: VecDeque::from([BtStage106Regs{r0:0xCC9E,..Default::default()}]),
                r_4bc44: VecDeque::from([BtStage106Regs{r0:0x41,r1:0x42,r2:0x43,r3:0x4444}]),
                r_6f246: VecDeque::from([BtStage106Regs{r0:0x51,r1:0x52,r2:0x53,r3:0x5555}]),
                r_414a0: VecDeque::from([BtStage106Regs{r0:0x414A0,..Default::default()}]),
                e: Vec::new(),
            }
        }
    }

    impl B {
        fn pop(q: &mut VecDeque<BtStage106Regs>) -> BtStage106Regs {
            q.pop_front().unwrap()
        }
    }

    impl BtStage106Backend for B {
        fn read_object_halfword(&mut self,o:u32,off:u32)->u16{
            let v=match off{0xec=>self.ec.pop_front().unwrap(),0x64=>self.h64.pop_front().unwrap(),_=>panic!()};
            self.e.push(E::Read("h",o.wrapping_add(off),u32::from(v)));v
        }
        fn write_object_halfword(&mut self,o:u32,off:u32,v:u16){
            self.e.push(E::Write("h",o,off,u32::from(v)));
        }
        fn read_object_byte(&mut self,o:u32,off:u32)->u8{
            let v=match off{0xa7=>self.a7,0x1f=>self.b1f.pop_front().unwrap(),0xeb=>self.beb.pop_front().unwrap(),_=>panic!()};
            self.e.push(E::Read("b",o.wrapping_add(off),u32::from(v)));v
        }
        fn write_object_byte(&mut self,o:u32,off:u32,v:u8){
            self.e.push(E::Write("b",o,off,u32::from(v)));
        }
        fn boundary_3ccdc(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage106Regs{
            self.e.push(E::Call("3ccdc",a,b,c,d));Self::pop(&mut self.r_3ccdc)
        }
        fn tail_3cc9e(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage106Regs{
            self.e.push(E::Call("3cc9e",a,b,c,d));Self::pop(&mut self.r_3cc9e)
        }
        fn boundary_4bc44(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage106Regs{
            self.e.push(E::Call("4bc44",a,b,c,d));Self::pop(&mut self.r_4bc44)
        }
        fn boundary_6f246(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage106Regs{
            self.e.push(E::Call("6f246",a,b,c,d));Self::pop(&mut self.r_6f246)
        }
        fn tail_414a0(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage106Regs{
            self.e.push(E::Call("414a0",a,b,c,d));Self::pop(&mut self.r_414a0)
        }
    }

    #[test]
    fn zero_mask_zero_high_local_return_sets_bit3_and_ignores_incoming_volatiles(){
        let mut b=B::default();
        b.ec=VecDeque::from([0]);
        b.a7=0x1f;
        b.b1f=VecDeque::from([0x40]);
        b.beb=VecDeque::from([0]);
        assert_eq!(bt_stage106_object_dispatch(0x1000,0xaaaa,0xbbbb,0xcccc,&mut b),0x1000);
        assert!(b.e.contains(&E::Write("b",0x1000,0x1f,0x48)));
        assert!(!b.e.iter().any(|e|matches!(e,E::Call(..))));
    }

    #[test]
    fn zero_mask_zero_high_first_tail_uses_raw_eb_and_zero_r1_r2(){
        let mut b=B::default();
        b.ec=VecDeque::from([0]);
        b.a7=0x01;
        b.beb=VecDeque::from([0x27]);
        assert_eq!(bt_stage106_object_dispatch(0x1000,9,8,7,&mut b),0xCC9E);
        assert!(b.e.contains(&E::Call("3cc9e",0x1000,0,0,0x27)));
    }

    #[test]
    fn zero_mask_nonzero_high_common_path_preserves_highmask_until_4bc44_and_rereads_ec(){
        let mut b=B::default();
        b.ec=VecDeque::from([0,0x1234,0x4567]);
        b.h64=VecDeque::from([0x89ab]);
        b.a7=0xA5;
        assert_eq!(bt_stage106_object_dispatch(0x1000,1,2,3,&mut b),0x414A0);
        assert!(b.e.contains(&E::Call("4bc44",0x1000,0xA0,0,0x1234)));
        assert!(b.e.contains(&E::Call("6f246",0,0x89ab,0x4567^STAGE106_BT_MASK,0x4444)));
        assert!(b.e.contains(&E::Call("414a0",0x1000,1,0,0x5555)));
        assert!(b.e.contains(&E::Write("h",0x1000,0x68,0x1234)));
    }

    #[test]
    fn nonzero_mask_zero_high_common_path_uses_full_initial_ec_and_mask(){
        let mut b=B::default();
        b.ec=VecDeque::from([0x3306,0x1111,0x2222]);
        b.h64=VecDeque::from([0x3333]);
        b.a7=0x01;
        let _=bt_stage106_object_dispatch(0x1000,1,2,3,&mut b);
        assert!(b.e.contains(&E::Call("4bc44",0x1000,0x3306,0x3306,0x1111)));
    }

    #[test]
    fn precheck_zero_return_common_path_preserves_returned_r1_r2_until_4bc44(){
        let mut b=B::default();
        b.ec=VecDeque::from([0x3306,0x7777,0x8888]);
        b.h64=VecDeque::from([0x9999]);
        b.a7=0xE1;
        b.r_3ccdc=VecDeque::from([BtStage106Regs{r0:0,r1:0xAAAA,r2:0xBBBB,r3:0xCCCC}]);
        let _=bt_stage106_object_dispatch(0x1000,1,2,3,&mut b);
        assert!(b.e.contains(&E::Call("3ccdc",0x1000,0x3306,0x3306,0xE1)));
        assert!(b.e.contains(&E::Call("4bc44",0x1000,0xAAAA,0xBBBB,0x7777)));
    }

    #[test]
    fn precheck_nonzero_return_eb10_returns_live_r0_locally(){
        let mut b=B::default();
        b.ec=VecDeque::from([0x3306]);
        b.a7=0xE1;
        b.beb=VecDeque::from([0x1F]);
        b.r_3ccdc=VecDeque::from([BtStage106Regs{r0:0xDEAD,r1:2,r2:3,r3:4}]);
        assert_eq!(bt_stage106_object_dispatch(0x1000,1,2,3,&mut b),0xDEAD);
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("3cc9e",..)|E::Call("4bc44",..))));
    }

    #[test]
    fn precheck_nonzero_return_second_tail_uses_live_r2_and_masked_eb(){
        let mut b=B::default();
        b.ec=VecDeque::from([0x3306]);
        b.a7=0xE1;
        b.beb=VecDeque::from([0x3F]);
        b.r_3ccdc=VecDeque::from([BtStage106Regs{r0:0xDEAD,r1:0xAAAA,r2:0xBEEF,r3:0xCCCC}]);
        assert_eq!(bt_stage106_object_dispatch(0x1000,1,2,3,&mut b),0xCC9E);
        assert!(b.e.contains(&E::Call("3cc9e",0x1000,1,0xBEEF,0x30)));
    }

    #[test]
    fn provenance_constants_are_exact(){
        assert_eq!(STAGE106_CURRENT_BT_OBJECT_DISPATCH_ADDR,0x16D90C);
        assert_eq!(STAGE106_LEGACY_BT_OBJECT_DISPATCH_ADDR,0x16AA10);
        assert_eq!(STAGE106_CURRENT_BODY_LEN,142);
        assert_eq!(STAGE106_BT_MASK,0x3306);
        assert_eq!(STAGE106_BT_PRECHECK_BOUNDARY,0x3CCDC);
        assert_eq!(STAGE106_BT_FIRST_TAIL,0x3CC9E);
        assert_eq!(STAGE106_BT_COMMON_BOUNDARY,0x4BC44);
        assert_eq!(STAGE106_BT_XOR_BOUNDARY,0x6F246);
        assert_eq!(STAGE106_BT_FINAL_TAIL,0x414A0);
    }
}


pub const STAGE107_CURRENT_BT_TAIL_DISPATCH_ADDR: u32 = 0x0016_D99C;
pub const STAGE107_LEGACY_BT_TAIL_DISPATCH_ADDR: u32 = 0x0016_AAA0;
pub const STAGE107_BT_GLOBAL_BASE_ADDR: u32 = 0x0020_8830;
pub const STAGE107_BT_FIRST_BOUNDARY: u32 = 0x0003_CCDC;
pub const STAGE107_BT_SECOND_BOUNDARY: u32 = 0x0003_CC9E;
pub const STAGE107_BT_SPECIAL_TAIL: u32 = 0x0003_C3B0;
pub const STAGE107_BT_DEFAULT_TAIL: u32 = 0x0002_EC18;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage107Regs {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
}

pub trait BtStage107Backend {
    fn read_object_byte(&mut self, object: u32, offset: u32) -> u8;
    fn read_object_halfword(&mut self, object: u32, offset: u32) -> u16;
    fn read_object_word(&mut self, object: u32, offset: u32) -> u32;
    fn read_global_word4(&mut self) -> u32;

    fn boundary_3ccdc(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage107Regs;
    fn boundary_3cc9e(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage107Regs;
    fn tail_3c3b0(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage107Regs;
    fn tail_2ec18(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) -> BtStage107Regs;
}

/// Exact register-state model of current `0x16D99C..0x16D9E4`.
///
/// The entry overwrites incoming R3 with object byte +0x1D shifted left by
/// twenty-four, while incoming R1/R2 remain live until overwritten by the
/// exact path. Both terminal transfers are tail calls; their R0 is final.
pub fn bt_stage107_tail_dispatch<B: BtStage107Backend>(
    object: u32,
    incoming_r1: u32,
    incoming_r2: u32,
    _incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let first_byte1d = backend.read_object_byte(object, 0x1D);
    let mut regs = BtStage107Regs {
        r0: object,
        r1: incoming_r1,
        r2: incoming_r2,
        r3: u32::from(first_byte1d) << 24,
    };

    if first_byte1d & 0x80 != 0 {
        let halfword_ec = backend.read_object_halfword(object, 0xEC);
        regs.r1 = u32::from(halfword_ec);
        regs = backend.boundary_3ccdc(regs.r0, regs.r1, regs.r2, regs.r3);
        regs.r1 = regs.r0;
        if regs.r0 == 1 {
            regs.r0 = object;
            regs = backend.boundary_3cc9e(regs.r0, regs.r1, regs.r2, regs.r3);
        }
    }

    let word38 = backend.read_object_word(object, 0x38);
    regs.r3 = word38;
    regs.r1 = word38 << 28;

    if word38 & 8 != 0 {
        let fresh_byte1d = backend.read_object_byte(object, 0x1D);
        regs.r3 = u32::from(fresh_byte1d);
        regs.r2 = u32::from(fresh_byte1d) << 24;

        if fresh_byte1d & 0x80 == 0 {
            let global_word4 = backend.read_global_word4();
            regs.r3 = global_word4 << 20;
            if global_word4 & (1 << 11) != 0 {
                regs.r0 = object;
                regs.r1 = 1;
                return backend.tail_3c3b0(regs.r0, regs.r1, regs.r2, regs.r3).r0;
            }
        }
    }

    regs.r0 = object;
    backend.tail_2ec18(regs.r0, regs.r1, regs.r2, regs.r3).r0
}

#[cfg(test)]
mod stage107_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum E {
        Read(&'static str, u32, u32),
        Call(&'static str, u32, u32, u32, u32),
    }

    struct B {
        byte1d: VecDeque<u8>,
        halfword_ec: u16,
        word38: u32,
        global_word4: u32,
        returns_3ccdc: VecDeque<BtStage107Regs>,
        returns_3cc9e: VecDeque<BtStage107Regs>,
        returns_3c3b0: VecDeque<BtStage107Regs>,
        returns_2ec18: VecDeque<BtStage107Regs>,
        e: Vec<E>,
    }

    impl Default for B {
        fn default() -> Self {
            Self {
                byte1d: VecDeque::from([0]),
                halfword_ec: 0x1234,
                word38: 0,
                global_word4: 0,
                returns_3ccdc: VecDeque::from([BtStage107Regs {
                    r0: 2, r1: 0x11, r2: 0x22, r3: 0x33,
                }]),
                returns_3cc9e: VecDeque::from([BtStage107Regs {
                    r0: 3, r1: 0x44, r2: 0x55, r3: 0x66,
                }]),
                returns_3c3b0: VecDeque::from([BtStage107Regs {
                    r0: 0xC3B0, ..Default::default()
                }]),
                returns_2ec18: VecDeque::from([BtStage107Regs {
                    r0: 0xEC18, ..Default::default()
                }]),
                e: Vec::new(),
            }
        }
    }

    impl B {
        fn pop(q: &mut VecDeque<BtStage107Regs>) -> BtStage107Regs {
            q.pop_front().unwrap()
        }
    }

    impl BtStage107Backend for B {
        fn read_object_byte(&mut self, object: u32, offset: u32) -> u8 {
            assert_eq!(offset, 0x1D);
            let v = self.byte1d.pop_front().unwrap();
            self.e.push(E::Read("byte1d", object, u32::from(v)));
            v
        }
        fn read_object_halfword(&mut self, object: u32, offset: u32) -> u16 {
            assert_eq!(offset, 0xEC);
            self.e.push(E::Read("half_ec", object, u32::from(self.halfword_ec)));
            self.halfword_ec
        }
        fn read_object_word(&mut self, object: u32, offset: u32) -> u32 {
            assert_eq!(offset, 0x38);
            self.e.push(E::Read("word38", object, self.word38));
            self.word38
        }
        fn read_global_word4(&mut self) -> u32 {
            self.e.push(E::Read("global4", STAGE107_BT_GLOBAL_BASE_ADDR + 4, self.global_word4));
            self.global_word4
        }
        fn boundary_3ccdc(&mut self, a:u32,b:u32,c:u32,d:u32)->BtStage107Regs{
            self.e.push(E::Call("3ccdc",a,b,c,d)); Self::pop(&mut self.returns_3ccdc)
        }
        fn boundary_3cc9e(&mut self, a:u32,b:u32,c:u32,d:u32)->BtStage107Regs{
            self.e.push(E::Call("3cc9e",a,b,c,d)); Self::pop(&mut self.returns_3cc9e)
        }
        fn tail_3c3b0(&mut self, a:u32,b:u32,c:u32,d:u32)->BtStage107Regs{
            self.e.push(E::Call("3c3b0",a,b,c,d)); Self::pop(&mut self.returns_3c3b0)
        }
        fn tail_2ec18(&mut self, a:u32,b:u32,c:u32,d:u32)->BtStage107Regs{
            self.e.push(E::Call("2ec18",a,b,c,d)); Self::pop(&mut self.returns_2ec18)
        }
    }

    #[test]
    fn initial_bit7_clear_keeps_incoming_r2_but_overwrites_r1_r3_at_word38_gate() {
        let mut b = B::default();
        b.byte1d = VecDeque::from([0x12]);
        b.word38 = 5;
        assert_eq!(bt_stage107_tail_dispatch(0x1000,0xAAAA,0xBBBB,0xCCCC,&mut b),0xEC18);
        assert!(!b.e.iter().any(|e| matches!(e,E::Call("3ccdc",..)|E::Call("3cc9e",..))));
        assert!(b.e.contains(&E::Call("2ec18",0x1000,0x5000_0000,0xBBBB,5)));
    }

    #[test]
    fn first_boundary_nonone_preserves_its_live_r2_until_word38_default_tail() {
        let mut b = B::default();
        b.byte1d = VecDeque::from([0x80]);
        b.word38 = 2;
        b.returns_3ccdc = VecDeque::from([BtStage107Regs{r0:7,r1:8,r2:0xCAFE,r3:0xDEAD}]);
        assert_eq!(bt_stage107_tail_dispatch(0x1000,2,3,4,&mut b),0xEC18);
        assert!(b.e.contains(&E::Call("3ccdc",0x1000,0x1234,3,0x8000_0000)));
        assert!(!b.e.iter().any(|e| matches!(e,E::Call("3cc9e",..))));
        assert!(b.e.contains(&E::Call("2ec18",0x1000,0x2000_0000,0xCAFE,2)));
    }

    #[test]
    fn exact_one_first_return_calls_second_and_its_r2_reaches_bit3clear_default_tail() {
        let mut b = B::default();
        b.byte1d = VecDeque::from([0x80]);
        b.word38 = 0;
        b.returns_3ccdc = VecDeque::from([BtStage107Regs{r0:1,r1:8,r2:0x1111,r3:0x2222}]);
        b.returns_3cc9e = VecDeque::from([BtStage107Regs{r0:9,r1:0x33,r2:0x4444,r3:0x5555}]);
        let _ = bt_stage107_tail_dispatch(0x1000,2,3,4,&mut b);
        assert!(b.e.contains(&E::Call("3cc9e",0x1000,1,0x1111,0x2222)));
        assert!(b.e.contains(&E::Call("2ec18",0x1000,0,0x4444,0)));
    }

    #[test]
    fn bit3set_fresh_bit7set_replaces_r2_r3_before_default_tail() {
        let mut b = B::default();
        b.byte1d = VecDeque::from([0x00,0xAB]);
        b.word38 = 8;
        let _ = bt_stage107_tail_dispatch(0x1000,2,0x9999,4,&mut b);
        assert!(b.e.contains(&E::Call("2ec18",0x1000,0x8000_0000,0xAB00_0000,0xAB)));
        assert!(!b.e.iter().any(|e| matches!(e,E::Read("global4",..))));
    }

    #[test]
    fn bit3set_fresh_clear_global_bit11clear_uses_shifted_global_r3_in_default_tail() {
        let mut b = B::default();
        b.byte1d = VecDeque::from([0x00,0x21]);
        b.word38 = 8;
        b.global_word4 = 0x123;
        let _ = bt_stage107_tail_dispatch(0x1000,2,3,4,&mut b);
        assert!(b.e.contains(&E::Call("2ec18",0x1000,0x8000_0000,0x2100_0000,0x1230_0000)));
    }

    #[test]
    fn special_condition_uses_literal_one_and_shifted_fresh_byte_and_global() {
        let mut b = B::default();
        b.byte1d = VecDeque::from([0x00,0x21]);
        b.word38 = 8;
        b.global_word4 = 0x812;
        assert_eq!(bt_stage107_tail_dispatch(0x1000,2,3,4,&mut b),0xC3B0);
        assert!(b.e.contains(&E::Call("3c3b0",0x1000,1,0x2100_0000,0x8120_0000)));
        assert!(!b.e.iter().any(|e| matches!(e,E::Call("2ec18",..))));
    }

    #[test]
    fn byte1d_is_a_true_reread_after_optional_call_chain() {
        let mut b = B::default();
        b.byte1d = VecDeque::from([0x80,0x01]);
        b.word38 = 8;
        b.global_word4 = 0x800;
        b.returns_3ccdc = VecDeque::from([BtStage107Regs{r0:2,r1:3,r2:4,r3:5}]);
        let _ = bt_stage107_tail_dispatch(0x1000,2,3,4,&mut b);
        assert!(b.e.contains(&E::Call("3ccdc",0x1000,0x1234,3,0x8000_0000)));
        assert!(b.e.contains(&E::Call("3c3b0",0x1000,1,0x0100_0000,0x8000_0000)));
    }

    #[test]
    fn provenance_constants_are_exact() {
        assert_eq!(STAGE107_CURRENT_BT_TAIL_DISPATCH_ADDR,0x16D99C);
        assert_eq!(STAGE107_LEGACY_BT_TAIL_DISPATCH_ADDR,0x16AAA0);
        assert_eq!(STAGE107_BT_GLOBAL_BASE_ADDR,0x208830);
        assert_eq!(STAGE107_BT_FIRST_BOUNDARY,0x3CCDC);
        assert_eq!(STAGE107_BT_SECOND_BOUNDARY,0x3CC9E);
        assert_eq!(STAGE107_BT_SPECIAL_TAIL,0x3C3B0);
        assert_eq!(STAGE107_BT_DEFAULT_TAIL,0x2EC18);
    }
}
pub const STAGE108_CURRENT_BT_DISPATCH_ADDR: u32 = 0x0016_D9E8;
pub const STAGE108_LEGACY_BT_DISPATCH_ADDR: u32 = 0x0016_AAEC;
pub const STAGE108_GATE_WORD_ADDR: u32 = 0x0020_90CC;
pub const STAGE108_FINAL_FLAG_BASE_ADDR: u32 = 0x0020_8830;

pub const STAGE108_BT_FIRST_BOUNDARY: u32 = 0x0003_CCDC;
pub const STAGE108_BT_OPTIONAL_BOUNDARY: u32 = 0x0003_CC9E;
pub const STAGE108_BT_STAGE106_TAIL: u32 = 0x0004_14A0;
pub const STAGE108_BT_WORD34_BOUNDARY: u32 = 0x0002_EB58;
pub const STAGE108_BT_ZERO_BOUNDARY: u32 = 0x0006_E9E0;
pub const STAGE108_BT_OBJECT_A_BOUNDARY: u32 = 0x0004_BF0C;
pub const STAGE108_BT_OBJECT_B_BOUNDARY: u32 = 0x0003_7158;
pub const STAGE108_BT_GATE_OBJECT_BOUNDARY: u32 = 0x0003_34F8;
pub const STAGE108_BT_GATE_FOLLOWUP_BOUNDARY: u32 = 0x0003_BC94;
pub const STAGE108_BT_WORD0_BOUNDARY: u32 = 0x0002_E678;
pub const STAGE108_BT_FINAL_PROBE_BOUNDARY: u32 = 0x0005_8488;
pub const STAGE108_BT_FINAL_TAIL: u32 = 0x0005_833C;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage108Regs {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
}

pub trait BtStage108Backend {
    fn read_object_halfword(&mut self, object: u32, offset: u32) -> u16;
    fn read_object_word(&mut self, object: u32, offset: u32) -> u32;
    fn read_object_byte(&mut self, object: u32, offset: u32) -> u8;
    fn write_object_byte(&mut self, object: u32, offset: u32, value: u8);
    fn read_gate_word(&mut self) -> u32;
    fn read_final_flag_word(&mut self) -> u32;

    fn boundary_3ccdc(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_3cc9e(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_414a0(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_2eb58(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_6e9e0(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_4bf0c(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_37158(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_334f8(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_3bc94(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_2e678(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn boundary_58488(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
    fn tail_5833c(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage108Regs;
}

pub fn bt_stage108_dispatch<B: BtStage108Backend>(
    object:u32,
    _incoming_r1:u32,
    incoming_r2:u32,
    incoming_r3:u32,
    backend:&mut B,
)->u32{
    let object_ec = backend.read_object_halfword(object, 0xEC);
    let saved_word0 = backend.read_object_word(object, 0);
    let mut regs = backend.boundary_3ccdc(object, u32::from(object_ec), incoming_r2, incoming_r3);

    regs.r1 = regs.r0;
    if regs.r0 == 1 {
        let object_1d = backend.read_object_byte(object, 0x1D);
        regs.r0 = u32::from(object_1d) << 24;
        regs.r3 = regs.r0;
        if object_1d & 0x80 == 0 {
            regs.r0 = object;
            regs = backend.boundary_3cc9e(regs.r0, regs.r1, regs.r2, regs.r3);
        }
    }

    regs.r2 = 0;
    regs.r1 = 1;
    regs.r0 = object;
    regs = backend.boundary_414a0(regs.r0, regs.r1, regs.r2, regs.r3);

    let object_34 = backend.read_object_word(object, 0x34);
    regs.r3 = object_34;
    regs.r2 = object_34.wrapping_shl(21);
    if object_34 & (1 << 10) != 0 {
        regs.r0 = object;
        regs = backend.boundary_2eb58(regs.r0, regs.r1, regs.r2, regs.r3);
    }

    regs.r1 = saved_word0;
    regs.r0 = 0;
    regs = backend.boundary_6e9e0(regs.r0, regs.r1, regs.r2, regs.r3);
    regs.r0 = object;
    regs = backend.boundary_4bf0c(regs.r0, regs.r1, regs.r2, regs.r3);
    regs.r0 = object;
    regs = backend.boundary_37158(regs.r0, regs.r1, regs.r2, regs.r3);

    let object_1c = backend.read_object_byte(object, 0x1C);
    regs.r2 = 8;
    regs.r3 = u32::from((object_1c & 0x07) | 0x40);
    backend.write_object_byte(object, 0x1C, regs.r3 as u8);

    let gate_word = backend.read_gate_word();
    regs.r3 = gate_word;
    if gate_word == 1 {
        regs.r0 = object;
        regs = backend.boundary_334f8(regs.r0, regs.r1, regs.r2, regs.r3);
        if regs.r0 == 1 {
            regs = backend.boundary_3bc94(regs.r0, regs.r1, regs.r2, regs.r3);
        }
    }

    regs.r0 = saved_word0;
    regs = backend.boundary_2e678(regs.r0, regs.r1, regs.r2, regs.r3);

    let final_word = backend.read_final_flag_word();
    regs.r3 = final_word.wrapping_shl(31);
    if final_word & 1 == 0 {
        return regs.r0;
    }

    regs = backend.boundary_58488(regs.r0, regs.r1, regs.r2, regs.r3);
    if regs.r0 == 0 {
        return 0;
    }

    regs.r0 = 0;
    let tail = backend.tail_5833c(regs.r0, regs.r1, regs.r2, incoming_r3);
    tail.r0
}

#[cfg(test)]
mod stage108_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    #[derive(Clone,Debug,PartialEq,Eq)]
    enum E { Call(&'static str,u32,u32,u32,u32), Read(&'static str,u32,u32), Write(&'static str,u32,u32) }

    struct B {
        ec:u16, word0:u32, b1d:u8, word34:u32, b1c:u8, gate:u32, final_word:u32,
        q3ccdc:VecDeque<BtStage108Regs>, q3cc9e:VecDeque<BtStage108Regs>, q414a0:VecDeque<BtStage108Regs>,
        q2eb58:VecDeque<BtStage108Regs>, q6e9e0:VecDeque<BtStage108Regs>, q4bf0c:VecDeque<BtStage108Regs>,
        q37158:VecDeque<BtStage108Regs>, q334f8:VecDeque<BtStage108Regs>, q3bc94:VecDeque<BtStage108Regs>,
        q2e678:VecDeque<BtStage108Regs>, q58488:VecDeque<BtStage108Regs>, q5833c:VecDeque<BtStage108Regs>, e:Vec<E>,
    }
    impl Default for B {
        fn default()->Self{Self{
            ec:0x1234,word0:0xA000,b1d:0,word34:0,b1c:0xFF,gate:0,final_word:0,
            q3ccdc:VecDeque::from([BtStage108Regs{r0:2,r1:0x11,r2:0x12,r3:0x13}]),
            q3cc9e:VecDeque::from([BtStage108Regs{r0:0x21,r1:0x22,r2:0x23,r3:0x24}]),
            q414a0:VecDeque::from([BtStage108Regs{r0:0x31,r1:0x32,r2:0x33,r3:0x34}]),
            q2eb58:VecDeque::from([BtStage108Regs{r0:0x41,r1:0x42,r2:0x43,r3:0x44}]),
            q6e9e0:VecDeque::from([BtStage108Regs{r0:0x51,r1:0x52,r2:0x53,r3:0x54}]),
            q4bf0c:VecDeque::from([BtStage108Regs{r0:0x61,r1:0x62,r2:0x63,r3:0x64}]),
            q37158:VecDeque::from([BtStage108Regs{r0:0x71,r1:0x72,r2:0x73,r3:0x74}]),
            q334f8:VecDeque::from([BtStage108Regs{r0:2,r1:0x82,r2:0x83,r3:0x84}]),
            q3bc94:VecDeque::from([BtStage108Regs{r0:0x91,r1:0x92,r2:0x93,r3:0x94}]),
            q2e678:VecDeque::from([BtStage108Regs{r0:0xCAFE,r1:0xA2,r2:0xA3,r3:0xA4}]),
            q58488:VecDeque::from([BtStage108Regs{r0:0,r1:0xB2,r2:0xB3,r3:0xB4}]),
            q5833c:VecDeque::from([BtStage108Regs{r0:0xBEEF,..Default::default()}]), e:Vec::new()
        }}
    }
    impl B { fn pop(q:&mut VecDeque<BtStage108Regs>)->BtStage108Regs{q.pop_front().unwrap()} }
    impl BtStage108Backend for B {
        fn read_object_halfword(&mut self,o:u32,off:u32)->u16{self.e.push(E::Read("h",o+off,u32::from(self.ec)));self.ec}
        fn read_object_word(&mut self,o:u32,off:u32)->u32{let v=if off==0{self.word0}else{self.word34};self.e.push(E::Read("w",o+off,v));v}
        fn read_object_byte(&mut self,o:u32,off:u32)->u8{let v=if off==0x1d{self.b1d}else{self.b1c};self.e.push(E::Read("b",o+off,u32::from(v)));v}
        fn write_object_byte(&mut self,o:u32,off:u32,v:u8){self.e.push(E::Write("b",o+off,u32::from(v)))}
        fn read_gate_word(&mut self)->u32{self.e.push(E::Read("gate",STAGE108_GATE_WORD_ADDR,self.gate));self.gate}
        fn read_final_flag_word(&mut self)->u32{self.e.push(E::Read("final",STAGE108_FINAL_FLAG_BASE_ADDR+8,self.final_word));self.final_word}
        fn boundary_3ccdc(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("3ccdc",a,b,c,d));Self::pop(&mut self.q3ccdc)}
        fn boundary_3cc9e(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("3cc9e",a,b,c,d));Self::pop(&mut self.q3cc9e)}
        fn boundary_414a0(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("414a0",a,b,c,d));Self::pop(&mut self.q414a0)}
        fn boundary_2eb58(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("2eb58",a,b,c,d));Self::pop(&mut self.q2eb58)}
        fn boundary_6e9e0(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("6e9e0",a,b,c,d));Self::pop(&mut self.q6e9e0)}
        fn boundary_4bf0c(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("4bf0c",a,b,c,d));Self::pop(&mut self.q4bf0c)}
        fn boundary_37158(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("37158",a,b,c,d));Self::pop(&mut self.q37158)}
        fn boundary_334f8(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("334f8",a,b,c,d));Self::pop(&mut self.q334f8)}
        fn boundary_3bc94(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("3bc94",a,b,c,d));Self::pop(&mut self.q3bc94)}
        fn boundary_2e678(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("2e678",a,b,c,d));Self::pop(&mut self.q2e678)}
        fn boundary_58488(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("58488",a,b,c,d));Self::pop(&mut self.q58488)}
        fn tail_5833c(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage108Regs{self.e.push(E::Call("5833c",a,b,c,d));Self::pop(&mut self.q5833c)}
    }

    #[test] fn first_nonone_skips_optional_and_preserves_first_r3_into_414a0(){
        let mut b=B::default();
        let _=bt_stage108_dispatch(0x1000,0x11,0x22,0x33,&mut b);
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("3cc9e",..))));
        assert!(b.e.contains(&E::Call("414a0",0x1000,1,0,0x13)));
    }
    #[test] fn exact_one_bit7_set_skips_optional_but_uses_shifted_byte_in_r3(){
        let mut b=B::default();b.b1d=0x81;b.q3ccdc=VecDeque::from([BtStage108Regs{r0:1,r1:2,r2:3,r3:4}]);
        let _=bt_stage108_dispatch(0x1000,0,0,0,&mut b);
        assert!(b.e.contains(&E::Call("414a0",0x1000,1,0,0x8100_0000)));
    }
    #[test] fn exact_one_bit7_clear_uses_optional_return_r3(){
        let mut b=B::default();b.b1d=1;b.q3ccdc=VecDeque::from([BtStage108Regs{r0:1,r1:2,r2:3,r3:4}]);
        let _=bt_stage108_dispatch(0x1000,0,0,0,&mut b);
        assert!(b.e.contains(&E::Call("3cc9e",0x1000,1,3,0x0100_0000)));
        assert!(b.e.contains(&E::Call("414a0",0x1000,1,0,0x24)));
    }
    #[test] fn word34_bit10_gates_boundary_and_chain_uses_saved_word0(){
        let mut b=B::default();b.word34=1<<10;
        let _=bt_stage108_dispatch(0x1000,0,0,0,&mut b);
        assert!(b.e.iter().any(|e|matches!(e,E::Call("2eb58",0x1000,0x32,r2,r3) if *r2==(1<<31) && *r3==(1<<10))));
        assert!(b.e.contains(&E::Call("6e9e0",0,0xA000,0x43,0x44)));
    }
    #[test] fn byte1c_rewrite_is_low3_or_0x40_and_gate_exact_one_controls_followup(){
        let mut b=B::default();b.b1c=0xFF;b.gate=1;b.q334f8=VecDeque::from([BtStage108Regs{r0:1,r1:0x82,r2:0x83,r3:0x84}]);
        let _=bt_stage108_dispatch(0x1000,0,0,0,&mut b);
        assert!(b.e.contains(&E::Write("b",0x101c,0x47)));
        assert!(b.e.contains(&E::Call("334f8",0x1000,0x72,8,1)));
        assert!(b.e.contains(&E::Call("3bc94",1,0x82,0x83,0x84)));
        assert!(b.e.contains(&E::Call("2e678",0xA000,0x92,0x93,0x94)));
    }
    #[test] fn final_flag_clear_returns_word0_boundary_result_without_probe(){
        let mut b=B::default();b.final_word=2;b.q2e678=VecDeque::from([BtStage108Regs{r0:0xCAFE,r1:1,r2:2,r3:3}]);
        assert_eq!(bt_stage108_dispatch(0x1000,0,0,0,&mut b),0xCAFE);
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("58488",..)|E::Call("5833c",..))));
    }
    #[test] fn final_probe_zero_returns_zero_and_nonzero_tails_with_restored_incoming_r3(){
        let mut b=B::default();b.final_word=1;b.q58488=VecDeque::from([BtStage108Regs{r0:0,r1:0xB2,r2:0xB3,r3:0xB4}]);
        assert_eq!(bt_stage108_dispatch(0x1000,0,0,0x4444,&mut b),0);
        let mut b=B::default();b.final_word=1;b.q58488=VecDeque::from([BtStage108Regs{r0:7,r1:0xB2,r2:0xB3,r3:0xB4}]);
        assert_eq!(bt_stage108_dispatch(0x1000,0,0,0x4444,&mut b),0xBEEF);
        assert!(b.e.contains(&E::Call("5833c",0,0xB2,0xB3,0x4444)));
    }
    #[test] fn provenance_constants_are_exact(){
        assert_eq!(STAGE108_CURRENT_BT_DISPATCH_ADDR,0x16D9E8);
        assert_eq!(STAGE108_LEGACY_BT_DISPATCH_ADDR,0x16AAEC);
        assert_eq!(STAGE108_BT_FIRST_BOUNDARY,0x3CCDC);
        assert_eq!(STAGE108_BT_FINAL_TAIL,0x5833C);
    }
}
pub const STAGE109_CURRENT_BT_COMPLEX_DISPATCH_ADDR: u32 = 0x0016_DA7C;
pub const STAGE109_CURRENT_BODY_LEN: u32 = 198;
pub const STAGE109_COUNT_BYTE_ADDR: u32 = 0x0020_33F0;
pub const STAGE109_COMPARE_BASE_ADDR: u32 = 0x0020_27FE;
pub const STAGE109_BT_FIRST_BOUNDARY: u32 = 0x0003_37AC;
pub const STAGE109_BT_SECOND_BOUNDARY: u32 = 0x0003_3808;
pub const STAGE109_BT_STATUS_TAIL: u32 = 0x0002_F756;
pub const STAGE109_BT_OBJECT28_BOUNDARY: u32 = 0x0003_36D0;
pub const STAGE109_BT_COMPARE_BOUNDARY: u32 = 0x000F_8CAC;
pub const STAGE109_BT_SPLIT_BOUNDARY: u32 = 0x0002_E644;
pub const STAGE109_BT_BYTE1E_TAIL: u32 = 0x0002_E3F8;
pub const STAGE109_BT_MODE_TAIL: u32 = 0x0002_E424;
pub const STAGE109_BT_FINAL_TAIL: u32 = 0x0002_CF64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage109Regs {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
}

pub trait BtStage109Backend {
    fn read_count_byte(&mut self) -> u8;
    fn read_object_byte(&mut self, object: u32, offset: u32) -> u8;
    fn write_object_byte(&mut self, object: u32, offset: u32, value: u8);
    fn read_object_word(&mut self, object: u32, offset: u32) -> u32;

    fn boundary_337ac(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage109Regs;
    fn boundary_33808(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage109Regs;
    fn tail_2f756(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage109Regs;
    fn boundary_336d0(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage109Regs;
    fn boundary_f8cac(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage109Regs;
    fn boundary_2e644(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage109Regs;
    fn tail_2e3f8(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage109Regs;
    fn tail_2e424(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage109Regs;
    fn tail_2cf64(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage109Regs;

    /// Models a second POP.W {R4,R5,R6,LR} after the local frame was already
    /// popped before the 0x2E644 call. The tail arguments are captured before
    /// this extra pop, but the stack/callee-saved side effect is observable.
    fn external_pop_after_local_frame(&mut self);
}

fn stage109_pop_for_exit<B: BtStage109Backend>(frame_popped: bool, backend: &mut B) {
    if frame_popped {
        backend.external_pop_after_local_frame();
    }
}

/// Exact register-state model of current `0x16DA7C..0x16DB42`.
///
/// `incoming_r4` is explicit because one branch restores the entry R4 via
/// POP.W before calling 0x2E644 and, if that opaque boundary returns, later
/// object-relative reads use the restored ambient R4 rather than saved R0.
/// A later POP on that returned path consumes caller-stack words; the backend
/// exposes that side effect rather than silently normalizing the stack shape.
pub fn bt_stage109_complex_dispatch<B: BtStage109Backend>(
    object: u32,
    incoming_r1: u32,
    incoming_r2: u32,
    incoming_r3: u32,
    incoming_r4: u32,
    backend: &mut B,
) -> u32 {
    let mut regs = backend.boundary_337ac(1, incoming_r1, incoming_r2, incoming_r3);
    let saved_first_r0 = regs.r0;

    regs.r0 = 1;
    regs = backend.boundary_33808(regs.r0, regs.r1, regs.r2, regs.r3);

    let count_plus_one = u32::from(backend.read_count_byte()).wrapping_add(1);
    regs.r0 = regs.r0.wrapping_add(saved_first_r0);
    let status = if regs.r0 == count_plus_one {
        Some(9u32)
    } else {
        let byte1c = backend.read_object_byte(object, 0x1C);
        if u32::from(byte1c & 0xF8) != 0x10 {
            Some(0x0B)
        } else {
            None
        }
    };

    if let Some(status) = status {
        return backend.tail_2f756(object, 0x33, 0, status).r0;
    }

    regs.r3 = 0x10;
    regs.r1 = object.wrapping_add(0x28);
    regs.r0 = object;
    regs = backend.boundary_336d0(regs.r0, regs.r1, regs.r2, regs.r3);
    if regs.r0 != 0 {
        return backend.tail_2f756(object, 0x33, 0, 0x0B).r0;
    }

    regs.r2 = 6;
    regs.r1 = object.wrapping_add(0x28);
    regs.r0 = STAGE109_COMPARE_BASE_ADDR;
    regs = backend.boundary_f8cac(regs.r0, regs.r1, regs.r2, regs.r3);
    regs.r2 = regs.r0;
    if regs.r0 == 0 {
        return backend.tail_2f756(object, 0x33, regs.r2, 0x0F).r0;
    }

    let byte1c = backend.read_object_byte(object, 0x1C);
    regs.r2 = 4;
    backend.write_object_byte(object, 0x1C, (byte1c & 0x07) | 0x20);

    let byte1f = backend.read_object_byte(object, 0x1F);
    regs.r3 = u32::from(byte1f);
    regs.r0 = u32::from(byte1f) << 30;

    let mut active_r4 = object;
    let mut frame_popped = false;

    if byte1f & 0x02 == 0 {
        regs.r1 = 1;
        regs.r0 = object;
        // POP.W {R4,R5,R6,LR} happens before the BL.
        active_r4 = incoming_r4;
        frame_popped = true;
    }
    regs = backend.boundary_2e644(regs.r0, regs.r1, regs.r2, regs.r3);

    let byte1e = backend.read_object_byte(active_r4, 0x1E);
    regs.r3 = u32::from(byte1e);
    regs.r1 = regs.r3 << 24;
    if byte1e & 0x80 == 0 {
        let tail_r0 = active_r4;
        let tail_r2 = regs.r2;
        let tail_r3 = regs.r3;
        stage109_pop_for_exit(frame_popped, backend);
        return backend.tail_2e3f8(tail_r0, 1, tail_r2, tail_r3).r0;
    }

    let byte1f_late = backend.read_object_byte(active_r4, 0x1F);
    regs.r3 = u32::from(byte1f_late);
    regs.r2 = regs.r3 << 31;

    if byte1f_late & 1 == 0 {
        let word38_first = backend.read_object_word(active_r4, 0x38);
        regs.r3 = word38_first;
        if (word38_first as i32) < 0 {
            let tail_r0 = active_r4;
            let tail_r2 = regs.r2;
            let tail_r3 = regs.r3;
            stage109_pop_for_exit(frame_popped, backend);
            return backend.tail_2e424(tail_r0, 1, tail_r2, tail_r3).r0;
        }
    }

    let byte_f7 = backend.read_object_byte(active_r4, 0xF7);
    regs.r3 = u32::from(byte_f7) << 24;
    if byte_f7 & 0x80 == 0 {
        let word38_second = backend.read_object_word(active_r4, 0x38);
        regs.r3 = word38_second;
        if (word38_second as i32) < 0 {
            let byte150 = backend.read_object_byte(active_r4, 0x150);
            regs.r3 = u32::from(byte150);
            if byte150 > 1 {
                let tail_r0 = active_r4;
                let tail_r2 = regs.r2;
                let tail_r3 = regs.r3;
                stage109_pop_for_exit(frame_popped, backend);
                return backend.tail_2e424(tail_r0, 2, tail_r2, tail_r3).r0;
            }
        }
    }

    let tail_r1 = active_r4;
    let tail_r2 = regs.r2;
    let tail_r3 = regs.r3;
    stage109_pop_for_exit(frame_popped, backend);
    backend.tail_2cf64(0, tail_r1, tail_r2, tail_r3).r0
}

#[cfg(test)]
mod stage109_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    #[derive(Clone,Debug,PartialEq,Eq)]
    enum E {
        Call(&'static str,u32,u32,u32,u32),
        Read(&'static str,u32,u32),
        Write(&'static str,u32,u32),
        ExternalPop,
    }

    struct B {
        count:u8,
        b1c:VecDeque<u8>,
        b1f:VecDeque<u8>,
        b1e:VecDeque<u8>,
        bf7:VecDeque<u8>,
        b150:VecDeque<u8>,
        w38:VecDeque<u32>,
        q337ac:VecDeque<BtStage109Regs>,
        q33808:VecDeque<BtStage109Regs>,
        q2f756:VecDeque<BtStage109Regs>,
        q336d0:VecDeque<BtStage109Regs>,
        qf8cac:VecDeque<BtStage109Regs>,
        q2e644:VecDeque<BtStage109Regs>,
        q2e3f8:VecDeque<BtStage109Regs>,
        q2e424:VecDeque<BtStage109Regs>,
        q2cf64:VecDeque<BtStage109Regs>,
        e:Vec<E>,
    }

    impl Default for B {
        fn default()->Self{Self{
            count:0,
            b1c:VecDeque::from([0x10,0x10]),
            b1f:VecDeque::from([0x02,0x01]),
            b1e:VecDeque::from([0x80]),
            bf7:VecDeque::from([0x80]),
            b150:VecDeque::from([0]),
            w38:VecDeque::from([0,0]),
            q337ac:VecDeque::from([BtStage109Regs{r0:0x10,r1:0x11,r2:0x12,r3:0x13}]),
            q33808:VecDeque::from([BtStage109Regs{r0:0x20,r1:0x21,r2:0x22,r3:0x23}]),
            q2f756:VecDeque::from([BtStage109Regs{r0:0xF756,..Default::default()}]),
            q336d0:VecDeque::from([BtStage109Regs{r0:0,r1:0x31,r2:0x32,r3:0x33}]),
            qf8cac:VecDeque::from([BtStage109Regs{r0:1,r1:0x41,r2:0x42,r3:0x43}]),
            q2e644:VecDeque::from([BtStage109Regs{r0:1,r1:0x51,r2:0x52,r3:0x53}]),
            q2e3f8:VecDeque::from([BtStage109Regs{r0:0xE3F4,..Default::default()}]),
            q2e424:VecDeque::from([BtStage109Regs{r0:0xE420,..Default::default()}]),
            q2cf64:VecDeque::from([BtStage109Regs{r0:0xCF60,..Default::default()}]),
            e:Vec::new(),
        }}
    }
    impl B { fn pop(q:&mut VecDeque<BtStage109Regs>)->BtStage109Regs{q.pop_front().unwrap()} }
    impl BtStage109Backend for B {
        fn read_count_byte(&mut self)->u8{self.e.push(E::Read("count",STAGE109_COUNT_BYTE_ADDR,u32::from(self.count)));self.count}
        fn read_object_byte(&mut self,o:u32,off:u32)->u8{
            let v=match off{
                0x1c=>self.b1c.pop_front().unwrap(),
                0x1f=>self.b1f.pop_front().unwrap(),
                0x1e=>self.b1e.pop_front().unwrap(),
                0xf7=>self.bf7.pop_front().unwrap(),
                0x150=>self.b150.pop_front().unwrap(),
                _=>panic!("bad byte offset"),
            };
            self.e.push(E::Read("byte",o.wrapping_add(off),u32::from(v)));v
        }
        fn write_object_byte(&mut self,o:u32,off:u32,v:u8){self.e.push(E::Write("byte",o.wrapping_add(off),u32::from(v)))}
        fn read_object_word(&mut self,o:u32,off:u32)->u32{assert_eq!(off,0x38);let v=self.w38.pop_front().unwrap();self.e.push(E::Read("word38",o.wrapping_add(off),v));v}
        fn boundary_337ac(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage109Regs{self.e.push(E::Call("337ac",a,b,c,d));Self::pop(&mut self.q337ac)}
        fn boundary_33808(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage109Regs{self.e.push(E::Call("33808",a,b,c,d));Self::pop(&mut self.q33808)}
        fn tail_2f756(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage109Regs{self.e.push(E::Call("2f756",a,b,c,d));Self::pop(&mut self.q2f756)}
        fn boundary_336d0(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage109Regs{self.e.push(E::Call("336d0",a,b,c,d));Self::pop(&mut self.q336d0)}
        fn boundary_f8cac(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage109Regs{self.e.push(E::Call("f8cac",a,b,c,d));Self::pop(&mut self.qf8cac)}
        fn boundary_2e644(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage109Regs{self.e.push(E::Call("2e644",a,b,c,d));Self::pop(&mut self.q2e644)}
        fn tail_2e3f8(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage109Regs{self.e.push(E::Call("2e3f8",a,b,c,d));Self::pop(&mut self.q2e3f8)}
        fn tail_2e424(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage109Regs{self.e.push(E::Call("2e424",a,b,c,d));Self::pop(&mut self.q2e424)}
        fn tail_2cf64(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage109Regs{self.e.push(E::Call("2cf64",a,b,c,d));Self::pop(&mut self.q2cf64)}
        fn external_pop_after_local_frame(&mut self){self.e.push(E::ExternalPop)}
    }

    #[test]
    fn count_match_status9_tail_shape(){
        let mut b=B::default();
        b.count=0x2f;
        b.q337ac=VecDeque::from([BtStage109Regs{r0:0x10,r1:2,r2:3,r3:4}]);
        b.q33808=VecDeque::from([BtStage109Regs{r0:0x20,r1:5,r2:6,r3:7}]);
        assert_eq!(bt_stage109_complex_dispatch(0x1000,9,8,7,6,&mut b),0xF756);
        assert!(b.e.contains(&E::Call("2f756",0x1000,0x33,0,9)));
    }

    #[test]
    fn mask_miss_status0b_without_object28_boundary(){
        let mut b=B::default(); b.count=0; b.b1c=VecDeque::from([0x08]);
        let _=bt_stage109_complex_dispatch(0x1000,1,2,3,4,&mut b);
        assert!(b.e.contains(&E::Call("2f756",0x1000,0x33,0,0x0b)));
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("336d0",..))));
    }

    #[test]
    fn compare_zero_status0f_and_live_r3_into_compare_boundary(){
        let mut b=B::default(); b.qf8cac=VecDeque::from([BtStage109Regs{r0:0,r1:9,r2:8,r3:7}]);
        b.q336d0=VecDeque::from([BtStage109Regs{r0:0,r1:0x31,r2:0x32,r3:0xABCD}]);
        let _=bt_stage109_complex_dispatch(0x1000,1,2,3,4,&mut b);
        assert!(b.e.contains(&E::Call("f8cac",STAGE109_COMPARE_BASE_ADDR,0x1028,6,0xABCD)));
        assert!(b.e.contains(&E::Call("2f756",0x1000,0x33,0,0x0f)));
    }

    #[test]
    fn byte1c_rewrite_and_bit1set_keeps_frame_for_2e644(){
        let mut b=B::default();
        b.b1c=VecDeque::from([0x10,0x17]); b.b1f=VecDeque::from([0x03,0x01]); b.b1e=VecDeque::from([0]);
        b.qf8cac=VecDeque::from([BtStage109Regs{r0:5,r1:0xAAAA,r2:0xBBBB,r3:0xCCCC}]);
        let _=bt_stage109_complex_dispatch(0x1000,1,2,3,0x4444,&mut b);
        assert!(b.e.contains(&E::Write("byte",0x101c,0x27)));
        assert!(b.e.contains(&E::Call("2e644",0xC000_0000,0xAAAA,4,3)));
        assert!(b.e.contains(&E::Call("2e3f8",0x1000,1,0x52,0)));
        assert!(!b.e.contains(&E::ExternalPop));
    }

    #[test]
    fn bit1clear_restores_ambient_r4_before_boundary_and_extra_pop_if_it_returns(){
        let mut b=B::default();
        b.b1c=VecDeque::from([0x10,0x10]); b.b1f=VecDeque::from([0x00]); b.b1e=VecDeque::from([0]);
        b.qf8cac=VecDeque::from([BtStage109Regs{r0:5,r1:0xAAAA,r2:0xBBBB,r3:0xCCCC}]);
        b.q2e644=VecDeque::from([BtStage109Regs{r0:7,r1:8,r2:0xCAFE,r3:9}]);
        let _=bt_stage109_complex_dispatch(0x1000,1,2,3,0x9000,&mut b);
        assert!(b.e.contains(&E::Call("2e644",0x1000,1,4,0)));
        assert!(b.e.contains(&E::Read("byte",0x901e,0)));
        assert!(b.e.contains(&E::ExternalPop));
        assert!(b.e.contains(&E::Call("2e3f8",0x9000,1,0xCAFE,0)));
    }

    #[test]
    fn first_signed_negative_path_tails_mode1_with_r2_zero_and_raw_word(){
        let mut b=B::default();
        b.b1c=VecDeque::from([0x10,0x10]); b.b1f=VecDeque::from([0x03,0x00]); b.b1e=VecDeque::from([0x80]); b.w38=VecDeque::from([0x8000_1234]);
        let _=bt_stage109_complex_dispatch(0x1000,1,2,3,4,&mut b);
        assert!(b.e.contains(&E::Call("2e424",0x1000,1,0,0x8000_1234)));
    }

    #[test]
    fn second_word_reread_and_byte150_mode2_preserve_bit0_shift_r2(){
        let mut b=B::default();
        b.b1c=VecDeque::from([0x10,0x10]); b.b1f=VecDeque::from([0x03,0x01]); b.b1e=VecDeque::from([0x80]);
        b.bf7=VecDeque::from([0x00]); b.w38=VecDeque::from([0x8000_0001]); b.b150=VecDeque::from([2]);
        let _=bt_stage109_complex_dispatch(0x1000,1,2,3,4,&mut b);
        assert!(b.e.contains(&E::Call("2e424",0x1000,2,0x8000_0000,2)));
    }

    #[test]
    fn final_tail_keeps_path_dependent_r2_r3(){
        let mut b=B::default();
        b.b1c=VecDeque::from([0x10,0x10]); b.b1f=VecDeque::from([0x03,0x01]); b.b1e=VecDeque::from([0x80]);
        b.bf7=VecDeque::from([0x80]);
        let _=bt_stage109_complex_dispatch(0x1000,1,2,3,4,&mut b);
        assert!(b.e.contains(&E::Call("2cf64",0,0x1000,0x8000_0000,0x8000_0000)));
    }

    #[test]
    fn provenance_constants_are_exact(){
        assert_eq!(STAGE109_CURRENT_BT_COMPLEX_DISPATCH_ADDR,0x16DA7C);
        assert_eq!(STAGE109_CURRENT_BODY_LEN,198);
        assert_eq!(STAGE109_BT_FIRST_BOUNDARY,0x337AC);
        assert_eq!(STAGE109_BT_SECOND_BOUNDARY,0x33808);
        assert_eq!(STAGE109_BT_SPLIT_BOUNDARY,0x2E644);
        assert_eq!(STAGE109_BT_FINAL_TAIL,0x2CF64);
    }
}
pub const STAGE110_CURRENT_BT_STATE_COUNTER_ADDR: u32 = 0x0016_DB4C;
pub const STAGE110_LEGACY_BT_STATE_COUNTER_ADDR: u32 = 0x0016_AB80;
pub const STAGE110_CURRENT_BODY_LEN: u32 = 136;
pub const STAGE110_WORD_MASK: u32 = 0x0300_7800;
pub const STAGE110_WORD_EXPECTED: u32 = 0x0300_1800;
pub const STAGE110_BT_NOTIFY_BOUNDARY: u32 = 0x0003_B04A;
pub const STAGE110_BT_BUFFER_BOUNDARY: u32 = 0x0000_3D24;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage110Regs {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
}

pub trait BtStage110Backend {
    fn read_input_byte(&mut self, input: u32, offset: u32) -> u8;
    fn read_input_word(&mut self, input: u32, offset: u32) -> u32;
    fn read_state_byte(&mut self, state: u32, offset: u32) -> u8;
    fn write_state_byte(&mut self, state: u32, offset: u32, value: u8);

    fn boundary_3b04a(&mut self, r0:u32,r1:u32,r2:u32,r3:u32) -> BtStage110Regs;
    fn boundary_3d24(&mut self, r0:u32,r1:u32,r2:u32,r3:u32) -> BtStage110Regs;
}

/// Exact register-state model of current `0x16DB4C..0x16DBD4`.
///
/// R4 saves incoming R1 as the state pointer. Local instructions deliberately
/// leave path-dependent caller-volatile R1-R3 live into `0x3B04A`; when state
/// byte +0x0E is zero, that boundary's R0 can become the final function return.
/// The `0x3D24` path overwrites R0-R2 but forwards raw state byte +0x0E in R3.
pub fn bt_stage110_state_counter<B: BtStage110Backend>(
    input: u32,
    state: u32,
    incoming_r2: u32,
    _incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let byte95 = backend.read_input_byte(input, 0x95);
    let mut regs = BtStage110Regs {
        r0: input,
        r1: state,
        r2: incoming_r2,
        r3: u32::from(byte95),
    };

    if byte95 == 2 {
        let word90 = backend.read_input_word(input, 0x90);
        regs.r2 = word90;
        regs.r3 = word90 & STAGE110_WORD_MASK;
        regs.r2 = STAGE110_WORD_EXPECTED;

        if regs.r3 == STAGE110_WORD_EXPECTED {
            let state16 = backend.read_state_byte(state, 0x16);
            regs.r3 = u32::from(state16).wrapping_add(1);
            backend.write_state_byte(state, 0x16, regs.r3 as u8);

            let state14 = backend.read_state_byte(state, 0x14);
            regs.r3 = u32::from(state14 | 0x10);
            backend.write_state_byte(state, 0x14, regs.r3 as u8);
        } else {
            let byte90 = backend.read_input_byte(input, 0x90);
            regs.r3 = u32::from((byte90 >> 3) & 0x0F);

            if regs.r3 > 2 {
                let byte92 = backend.read_input_byte(input, 0x92);
                regs.r2 = u32::from(byte92);
                regs.r3 = u32::from(byte92 & 0x03).wrapping_sub(1);

                if regs.r3 <= 1 {
                    regs.r2 = u32::from(byte92) << 29;
                    if (regs.r2 as i32) >= 0 {
                        let state14 = backend.read_state_byte(state, 0x14);
                        regs.r3 = u32::from(state14 | 0x04);
                        backend.write_state_byte(state, 0x14, regs.r3 as u8);
                    }
                }
            }
        }
    }

    let byte0f = backend.read_state_byte(state, 0x0F);
    regs.r3 = u32::from(byte0f);
    if byte0f != 0 {
        regs.r3 = 0;
        backend.write_state_byte(state, 0x0F, 0);
        regs.r0 = state;
        regs = backend.boundary_3b04a(regs.r0, regs.r1, regs.r2, regs.r3);
    }

    let byte0e = backend.read_state_byte(state, 0x0E);
    regs.r3 = u32::from(byte0e);
    if byte0e != 0 {
        backend.write_state_byte(state, 0x0E, 0);
        regs.r2 = 0x74;
        regs.r1 = 0;
        regs.r0 = state.wrapping_add(0x10);
        regs = backend.boundary_3d24(regs.r0, regs.r1, regs.r2, regs.r3);
        backend.write_state_byte(state, 1, 0);
        return regs.r0;
    }

    let state14 = backend.read_state_byte(state, 0x14);
    regs.r2 = u32::from(state14);
    regs.r3 = regs.r2 << 31;
    if state14 & 1 == 0 {
        let counter = backend.read_state_byte(state, 1);
        regs.r3 = u32::from(counter);
        let limit = u32::from(state14) >> 5;
        if regs.r3 < limit {
            regs.r3 = regs.r3.wrapping_add(1);
            backend.write_state_byte(state, 1, regs.r3 as u8);
        }
    }

    regs.r0
}

#[cfg(test)]
mod stage110_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum E {
        Read(&'static str,u32,u32),
        Write(&'static str,u32,u32),
        Call(&'static str,u32,u32,u32,u32),
    }

    struct B {
        in95:u8, word90:u32, in90:u8, in92:u8,
        state14:VecDeque<u8>, state16:u8, state0f:u8, state0e:u8, counter:u8,
        q3b04a:VecDeque<BtStage110Regs>, q3d24:VecDeque<BtStage110Regs>,
        e:Vec<E>,
    }

    impl Default for B {
        fn default()->Self { Self {
            in95:0, word90:0, in90:0, in92:0,
            state14:VecDeque::from([0]), state16:0, state0f:0, state0e:0, counter:0,
            q3b04a:VecDeque::from([BtStage110Regs{r0:0xB04A,r1:0xB1,r2:0xB2,r3:0xB3}]),
            q3d24:VecDeque::from([BtStage110Regs{r0:0x3D24,r1:0xD1,r2:0xD2,r3:0xD3}]),
            e:Vec::new(),
        }}
    }

    impl B {
        fn pop(q:&mut VecDeque<BtStage110Regs>)->BtStage110Regs { q.pop_front().unwrap() }
    }

    impl BtStage110Backend for B {
        fn read_input_byte(&mut self,o:u32,off:u32)->u8 {
            let v=match off {0x95=>self.in95,0x90=>self.in90,0x92=>self.in92,_=>panic!()};
            self.e.push(E::Read("ib",o.wrapping_add(off),u32::from(v))); v
        }
        fn read_input_word(&mut self,o:u32,off:u32)->u32 {
            assert_eq!(off,0x90); self.e.push(E::Read("iw",o.wrapping_add(off),self.word90)); self.word90
        }
        fn read_state_byte(&mut self,s:u32,off:u32)->u8 {
            let v=match off {
                0x14=>self.state14.pop_front().unwrap(),
                0x16=>self.state16,
                0x0f=>self.state0f,
                0x0e=>self.state0e,
                1=>self.counter,
                _=>panic!(),
            };
            self.e.push(E::Read("sb",s.wrapping_add(off),u32::from(v))); v
        }
        fn write_state_byte(&mut self,s:u32,off:u32,v:u8) {
            if off==1 { self.counter=v; }
            self.e.push(E::Write("sb",s.wrapping_add(off),u32::from(v)));
        }
        fn boundary_3b04a(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage110Regs {
            self.e.push(E::Call("3b04a",a,b,c,d)); Self::pop(&mut self.q3b04a)
        }
        fn boundary_3d24(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage110Regs {
            self.e.push(E::Call("3d24",a,b,c,d)); Self::pop(&mut self.q3d24)
        }
    }

    #[test]
    fn byte95_not_two_preserves_incoming_r2_and_byte95_r3_into_notify() {
        let mut b=B::default(); b.in95=7; b.state0f=1;
        assert_eq!(bt_stage110_state_counter(0x1000,0x2000,0xAAAA,0xBBBB,&mut b),0xB04A);
        assert!(b.e.contains(&E::Call("3b04a",0x2000,0x2000,0xAAAA,0)));
    }

    #[test]
    fn exact_word_match_increments_byte16_and_sets_bit4_with_expected_literal_live_in_r2() {
        let mut b=B::default(); b.in95=2; b.word90=STAGE110_WORD_EXPECTED; b.state16=0xFF;
        b.state14=VecDeque::from([0x21,0x31]); b.state0f=1;
        let _=bt_stage110_state_counter(0x1000,0x2000,3,4,&mut b);
        assert!(b.e.contains(&E::Write("sb",0x2016,0)));
        assert!(b.e.contains(&E::Write("sb",0x2014,0x31)));
        assert!(b.e.contains(&E::Call("3b04a",0x2000,0x2000,STAGE110_WORD_EXPECTED,0)));
    }

    #[test]
    fn alternate_gate_sets_bit2_and_preserves_shifted_byte92_in_r2() {
        let mut b=B::default(); b.in95=2; b.word90=0; b.in90=0x18; b.in92=0x02;
        b.state14=VecDeque::from([0x10,0x14]); b.state0f=1;
        let _=bt_stage110_state_counter(0x1000,0x2000,3,4,&mut b);
        assert!(b.e.contains(&E::Write("sb",0x2014,0x14)));
        assert!(b.e.contains(&E::Call("3b04a",0x2000,0x2000,0x4000_0000,0)));
    }

    #[test]
    fn alternate_gate_failure_leaves_raw_local_register_shape_for_notify() {
        let mut b=B::default(); b.in95=2; b.word90=0; b.in90=0x10; b.state0f=1;
        let _=bt_stage110_state_counter(0x1000,0x2000,3,4,&mut b);
        assert!(b.e.contains(&E::Call("3b04a",0x2000,0x2000,STAGE110_WORD_EXPECTED,0)));
    }

    #[test]
    fn notify_return_r0_is_final_when_byte0e_zero_and_bit0_suppresses_counter() {
        let mut b=B::default(); b.in95=9; b.state0f=1; b.state0e=0; b.state14=VecDeque::from([1]);
        assert_eq!(bt_stage110_state_counter(0x1000,0x2000,3,4,&mut b),0xB04A);
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("3d24",..))));
    }

    #[test]
    fn buffer_boundary_gets_raw_byte0e_in_r3_and_its_r0_is_final() {
        let mut b=B::default(); b.in95=9; b.state0f=1; b.state0e=0xA5;
        assert_eq!(bt_stage110_state_counter(0x1000,0x2000,3,4,&mut b),0x3D24);
        assert!(b.e.contains(&E::Call("3d24",0x2010,0,0x74,0xA5)));
        assert!(b.e.contains(&E::Write("sb",0x2001,0)));
    }

    #[test]
    fn counter_increments_only_below_state14_high3_limit() {
        let mut b=B::default(); b.in95=9; b.state14=VecDeque::from([0x60]); b.counter=2;
        assert_eq!(bt_stage110_state_counter(0x1000,0x2000,3,4,&mut b),0x1000);
        assert_eq!(b.counter,3);

        let mut b=B::default(); b.in95=9; b.state14=VecDeque::from([0x60]); b.counter=3;
        let _=bt_stage110_state_counter(0x1000,0x2000,3,4,&mut b);
        assert_eq!(b.counter,3);
    }

    #[test]
    fn provenance_constants_are_exact() {
        assert_eq!(STAGE110_CURRENT_BT_STATE_COUNTER_ADDR,0x16DB4C);
        assert_eq!(STAGE110_LEGACY_BT_STATE_COUNTER_ADDR,0x16AB80);
        assert_eq!(STAGE110_CURRENT_BODY_LEN,136);
        assert_eq!(STAGE110_WORD_MASK,0x03007800);
        assert_eq!(STAGE110_WORD_EXPECTED,0x03001800);
        assert_eq!(STAGE110_BT_NOTIFY_BOUNDARY,0x3B04A);
        assert_eq!(STAGE110_BT_BUFFER_BOUNDARY,0x3D24);
    }
}
pub const STAGE111_CURRENT_BT_RECORD_DISPATCH_ADDR: u32 = 0x0016_DBDC;
pub const STAGE111_LEGACY_BT_RECORD_DISPATCH_ADDR: u32 = 0x0016_AC10;
pub const STAGE111_CURRENT_BODY_LEN: u32 = 326;
pub const STAGE111_RECORD_STRIDE: u32 = 0x19;

pub const STAGE111_BT_FIRST_BOUNDARY: u32 = 0x0003_35AC;
pub const STAGE111_BT_RECORD_VALUE_BOUNDARY: u32 = 0x0003_A6CC;
pub const STAGE111_BT_NOTIFY_BOUNDARY: u32 = 0x0003_B04A;
pub const STAGE111_BT_BUFFER_BOUNDARY: u32 = 0x0000_3D24;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage111Regs {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
}

pub trait BtStage111Backend {
    fn read_byte(&mut self, addr: u32) -> u8;
    fn read_halfword(&mut self, addr: u32) -> u16;
    fn write_byte(&mut self, addr: u32, value: u8);
    fn write_halfword(&mut self, addr: u32, value: u16);

    fn boundary_335ac(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage111Regs;
    fn boundary_3a6cc(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage111Regs;
    fn boundary_3b04a(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage111Regs;
    fn boundary_3d24(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage111Regs;
}

fn stage111_record(state: u32, index: u8) -> u32 {
    state.wrapping_add(STAGE111_RECORD_STRIDE.wrapping_mul(u32::from(index)))
}

/// Exact register-state model of current `0x16DBDC..0x16DD22`.
///
/// The model preserves true rereads of object byte +0x90, path-dependent
/// record selection, and the full caller-volatile chain into the late opaque
/// boundaries. Incoming R3 is forwarded to the first boundary and may remain
/// observable through that boundary's return until local instructions replace it.
pub fn bt_stage111_record_dispatch<B: BtStage111Backend>(
    object: u32,
    state: u32,
    index_ptr: u32,
    incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let saved_object94 = backend.read_byte(object.wrapping_add(0x94));
    let object_a4 = backend.read_byte(object.wrapping_add(0xA4));
    let saved_state1 = backend.read_byte(state.wrapping_add(1));
    let index6 = backend.read_byte(index_ptr.wrapping_add(6));

    let mut regs = backend.boundary_335ac(
        u32::from(object_a4),
        state,
        index_ptr,
        incoming_r3,
    );
    if regs.r0 == 0 {
        return 0;
    }

    if saved_object94 == 2 {
        let object90_first = backend.read_byte(object.wrapping_add(0x90));
        regs.r3 = u32::from(object90_first);
        regs.r2 = regs.r3 << 24;

        if object90_first & 0x80 == 0 {
            let state14 = backend.read_byte(state.wrapping_add(0x14));
            regs.r3 = u32::from(state14 | 0x02);
            backend.write_byte(state.wrapping_add(0x14), regs.r3 as u8);
        }

        let object90_second = backend.read_byte(object.wrapping_add(0x90));
        regs.r3 = u32::from((object90_second >> 3) & 0x0F);
        if regs.r3 <= 2 {
            let state16 = backend.read_byte(state.wrapping_add(0x16));
            regs.r3 = u32::from(state16).wrapping_add(1);
            backend.write_byte(state.wrapping_add(0x16), regs.r3 as u8);
        }
    }

    if index6 != 0 {
        let index0 = backend.read_byte(index_ptr);
        regs.r3 = u32::from((index0 >> 3) & 0x0F);
        if regs.r3 > 2 {
            let index2 = backend.read_byte(index_ptr.wrapping_add(2));
            regs.r3 = u32::from(index2 & 0x03).wrapping_sub(1);
            if regs.r3 <= 1 {
                if saved_object94 == 2 {
                    let object91 = backend.read_byte(object.wrapping_add(0x91));
                    regs.r3 = u32::from(object91);
                    if object91 & 1 == 0 {
                        let record = stage111_record(state, saved_state1);
                        let v = backend.read_byte(record.wrapping_add(0x22));
                        regs.r3 = u32::from(v).wrapping_add(1);
                        backend.write_byte(record.wrapping_add(0x22), regs.r3 as u8);
                    } else {
                        regs = backend.boundary_3a6cc(regs.r0, regs.r1, regs.r2, regs.r3);
                        let record = stage111_record(state, saved_state1);
                        backend.write_byte(record.wrapping_add(0x28), regs.r0 as u8);
                        let signed113 = backend.read_byte(object.wrapping_add(0x113));
                        regs.r3 = (signed113 as i8 as i32) as u32;
                        backend.write_byte(record.wrapping_add(0x29), regs.r3 as u8);
                    }
                } else {
                    let record = stage111_record(state, saved_state1);
                    let v = backend.read_byte(record.wrapping_add(0x21));
                    regs.r3 = u32::from(v).wrapping_add(1);
                    backend.write_byte(record.wrapping_add(0x21), regs.r3 as u8);
                }
            }
        }

        regs.r1 = u32::from(backend.read_byte(object.wrapping_add(0x0F)));
        if regs.r1 == 0 {
            let state0 = backend.read_byte(state);
            regs.r3 = u32::from(state0);
            if state0 != 0 {
                let live_state1 = backend.read_byte(state.wrapping_add(1));
                let index4 = backend.read_byte(index_ptr.wrapping_add(4));
                regs.r0 = u32::from(index4);
                regs.r2 = STAGE111_RECORD_STRIDE;
                let record = stage111_record(state, live_state1);
                let object96 = backend.read_byte(object.wrapping_add(0x96));
                regs.r2 = u32::from(object96).wrapping_add(regs.r0);
                regs.r0 = u32::from(backend.read_halfword(record.wrapping_add(0x26)));
                regs.r2 = regs.r2.wrapping_add(regs.r0);
                backend.write_halfword(record.wrapping_add(0x26), regs.r2 as u16);
                backend.write_byte(state, 0);
            }
        }
    } else {
        let object0f = backend.read_byte(object.wrapping_add(0x0F));
        regs.r3 = u32::from(object0f);
        if object0f == 1 && saved_object94 != 2 {
            let record = stage111_record(state, saved_state1);
            let v = backend.read_byte(record.wrapping_add(0x23));
            regs.r3 = u32::from(v).wrapping_add(1);
            backend.write_byte(record.wrapping_add(0x23), regs.r3 as u8);

            let state0 = backend.read_byte(state);
            regs.r3 = u32::from(state0);
            if state0 != 0 {
                let h = backend.read_halfword(record.wrapping_add(0x26));
                regs.r3 = u32::from(h).wrapping_add(2);
                backend.write_halfword(record.wrapping_add(0x26), regs.r3 as u16);
                backend.write_byte(state, 0);
            }
        }
    }

    let object90_gate = backend.read_byte(object.wrapping_add(0x90));
    regs.r3 = u32::from((object90_gate >> 3) & 0x0F);
    if regs.r3 > 1 {
        return regs.r0;
    }

    let state0f = backend.read_byte(state.wrapping_add(0x0F));
    regs.r3 = u32::from(state0f);
    if state0f != 0 {
        backend.write_byte(state.wrapping_add(0x0F), 0);
        regs.r0 = state;
        regs.r3 = 0;
        regs = backend.boundary_3b04a(regs.r0, regs.r1, regs.r2, regs.r3);
    }

    let state0e = backend.read_byte(state.wrapping_add(0x0E));
    regs.r3 = u32::from(state0e);
    if state0e != 0 {
        backend.write_byte(state.wrapping_add(0x0E), 0);
        regs.r2 = 0x74;
        regs.r1 = 0;
        regs.r0 = state.wrapping_add(0x10);
        regs = backend.boundary_3d24(regs.r0, regs.r1, regs.r2, regs.r3);
        backend.write_byte(state.wrapping_add(1), 0);
        return regs.r0;
    }

    let state14 = backend.read_byte(state.wrapping_add(0x14));
    regs.r2 = u32::from(state14);
    regs.r3 = regs.r2 << 31;
    if state14 & 1 == 0 {
        let counter = backend.read_byte(state.wrapping_add(1));
        regs.r3 = u32::from(counter);
        if regs.r3 < (regs.r2 >> 5) {
            regs.r3 = regs.r3.wrapping_add(1);
            backend.write_byte(state.wrapping_add(1), regs.r3 as u8);
        }
    }

    regs.r0
}

#[cfg(test)]
mod stage111_tests {
    use super::*;
    use std::collections::{BTreeMap, VecDeque};
    use std::vec::Vec;

    #[derive(Clone,Debug,PartialEq,Eq)]
    enum E {
        Read8(u32,u8),
        Read16(u32,u16),
        Write8(u32,u8),
        Write16(u32,u16),
        Call(&'static str,u32,u32,u32,u32),
    }

    struct B {
        b8:BTreeMap<u32,VecDeque<u8>>,
        h16:BTreeMap<u32,VecDeque<u16>>,
        q335ac:VecDeque<BtStage111Regs>,
        q3a6cc:VecDeque<BtStage111Regs>,
        q3b04a:VecDeque<BtStage111Regs>,
        q3d24:VecDeque<BtStage111Regs>,
        e:Vec<E>,
    }

    impl Default for B {
        fn default()->Self {
            let mut b=Self{
                b8:BTreeMap::new(),h16:BTreeMap::new(),
                q335ac:VecDeque::from([BtStage111Regs{r0:0x5000,r1:0x11,r2:0x22,r3:0x33}]),
                q3a6cc:VecDeque::from([BtStage111Regs{r0:0x66,r1:0x61,r2:0x62,r3:0x63}]),
                q3b04a:VecDeque::from([BtStage111Regs{r0:0xB04A,r1:0x71,r2:0x72,r3:0x73}]),
                q3d24:VecDeque::from([BtStage111Regs{r0:0x3D24,r1:0x81,r2:0x82,r3:0x83}]),
                e:Vec::new(),
            };
            b.push8(0x1094,[0]);
            b.push8(0x10A4,[7]);
            b.push8(0x2001,[1,1,1]);
            b.push8(0x3006,[1]);
            b.push8(0x100F,[1,1]);
            b.push8(0x1090,[0x20,0x20,0x20]);
            b.push8(0x200F,[0]);
            b.push8(0x200E,[0]);
            b.push8(0x2014,[1]);
            b
        }
    }

    impl B {
        fn push8<const N:usize>(&mut self,a:u32,v:[u8;N]){self.b8.insert(a,VecDeque::from(v));}
        fn push16<const N:usize>(&mut self,a:u32,v:[u16;N]){self.h16.insert(a,VecDeque::from(v));}
        fn pop(q:&mut VecDeque<BtStage111Regs>)->BtStage111Regs{q.pop_front().unwrap()}
    }

    impl BtStage111Backend for B {
        fn read_byte(&mut self,a:u32)->u8{
            let q=self.b8.get_mut(&a).unwrap_or_else(||panic!("no byte at {a:#x}"));
            let v=q.pop_front().unwrap_or_else(||panic!("empty byte at {a:#x}"));
            self.e.push(E::Read8(a,v));v
        }
        fn read_halfword(&mut self,a:u32)->u16{
            let q=self.h16.get_mut(&a).unwrap_or_else(||panic!("no half at {a:#x}"));
            let v=q.pop_front().unwrap_or_else(||panic!("empty half at {a:#x}"));
            self.e.push(E::Read16(a,v));v
        }
        fn write_byte(&mut self,a:u32,v:u8){self.e.push(E::Write8(a,v));}
        fn write_halfword(&mut self,a:u32,v:u16){self.e.push(E::Write16(a,v));}
        fn boundary_335ac(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage111Regs{
            self.e.push(E::Call("335ac",a,b,c,d));Self::pop(&mut self.q335ac)
        }
        fn boundary_3a6cc(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage111Regs{
            self.e.push(E::Call("3a6cc",a,b,c,d));Self::pop(&mut self.q3a6cc)
        }
        fn boundary_3b04a(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage111Regs{
            self.e.push(E::Call("3b04a",a,b,c,d));Self::pop(&mut self.q3b04a)
        }
        fn boundary_3d24(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage111Regs{
            self.e.push(E::Call("3d24",a,b,c,d));Self::pop(&mut self.q3d24)
        }
    }

    #[test]
    fn zero_first_boundary_exits_immediately() {
        let mut b=B::default();
        b.q335ac=VecDeque::from([BtStage111Regs{r0:0,r1:1,r2:2,r3:3}]);
        assert_eq!(bt_stage111_record_dispatch(0x1000,0x2000,0x3000,0x4444,&mut b),0);
        assert!(b.e.contains(&E::Call("335ac",7,0x2000,0x3000,0x4444)));
        assert!(!b.e.iter().any(|e|matches!(e,E::Read8(0x1090,_))));
    }

    #[test]
    fn object94_two_has_true_byte90_rereads_and_state_flag_updates() {
        let mut b=B::default();
        b.push8(0x1094,[2]);
        b.push8(0x1090,[0x01,0x10,0x20]);
        b.push8(0x2014,[0x20,0x21]);
        b.push8(0x2016,[0xFF]);
        b.push8(0x3000,[0]);
        assert_eq!(bt_stage111_record_dispatch(0x1000,0x2000,0x3000,4,&mut b),0x5000);
        assert!(b.e.contains(&E::Write8(0x2014,0x22)));
        assert!(b.e.contains(&E::Write8(0x2016,0)));
        let reads:Vec<_>=b.e.iter().filter(|e|matches!(e,E::Read8(0x1090,_))).collect();
        assert_eq!(reads.len(),3);
    }

    #[test]
    fn index_nonzero_object94_two_bit0clear_increments_record22() {
        let mut b=B::default();
        b.push8(0x1094,[2]);
        b.push8(0x1090,[0x80,0x20,0x20]);
        b.push8(0x3000,[0x18]); b.push8(0x3002,[1]); b.push8(0x1091,[0]);
        b.push8(0x203B,[9]);
        b.push8(0x100F,[1]);
        let _=bt_stage111_record_dispatch(0x1000,0x2000,0x3000,4,&mut b);
        assert!(b.e.contains(&E::Write8(0x203B,10)));
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("3a6cc",..))));
    }

    #[test]
    fn index_nonzero_object94_two_bit0set_calls_boundary_and_stores_return_and_signed_byte() {
        let mut b=B::default();
        b.push8(0x1094,[2]);
        b.push8(0x1090,[0x80,0x20,0x20]);
        b.push8(0x3000,[0x18]); b.push8(0x3002,[2]); b.push8(0x1091,[1]);
        b.push8(0x1113,[0xFE]);
        b.push8(0x100F,[1]);
        b.q335ac=VecDeque::from([BtStage111Regs{r0:0x5000,r1:0xAAAA,r2:0xBBBB,r3:0xCCCC}]);
        let _=bt_stage111_record_dispatch(0x1000,0x2000,0x3000,4,&mut b);
        assert!(b.e.contains(&E::Call("3a6cc",0x5000,0xAAAA,0x8000_0000,1)));
        assert!(b.e.contains(&E::Write8(0x2041,0x66)));
        assert!(b.e.contains(&E::Write8(0x2042,0xFE)));
    }

    #[test]
    fn non_object94_two_record21_and_accumulation_use_saved_and_live_state1_separately() {
        let mut b=B::default();
        b.push8(0x1094,[1]);
        b.push8(0x3000,[0x18]); b.push8(0x3002,[1]); b.push8(0x3004,[5]);
        b.push8(0x203A,[7]);
        b.push8(0x100F,[0]);
        b.push8(0x2000,[1]);
        b.push8(0x2001,[1,2]);
        b.push8(0x1096,[6]);
        b.push16(0x2058,[10]);
        b.push8(0x1090,[0x20]);
        let _=bt_stage111_record_dispatch(0x1000,0x2000,0x3000,4,&mut b);
        assert!(b.e.contains(&E::Write8(0x203A,8)));
        assert!(b.e.contains(&E::Write16(0x2058,21)));
        assert!(b.e.contains(&E::Write8(0x2000,0)));
    }

    #[test]
    fn index6_zero_path_updates_record23_and_halfword_then_clears_state0() {
        let mut b=B::default();
        b.push8(0x1094,[1]);
        b.push8(0x3006,[0]);
        b.push8(0x100F,[1]);
        b.push8(0x203C,[4]);
        b.push8(0x2000,[1]);
        b.push16(0x203F,[0x10]);
        b.push8(0x1090,[0x20]);
        let _=bt_stage111_record_dispatch(0x1000,0x2000,0x3000,4,&mut b);
        assert!(b.e.contains(&E::Write8(0x203C,5)));
        assert!(b.e.contains(&E::Write16(0x203F,0x12)));
        assert!(b.e.contains(&E::Write8(0x2000,0)));
    }

    #[test]
    fn late_notify_preserves_live_r1_r2_and_can_supply_final_r0() {
        let mut b=B::default();
        b.push8(0x3006,[0]); b.push8(0x100F,[0]);
        b.push8(0x1090,[0x00]);
        b.push8(0x200F,[1]);
        b.push8(0x200E,[0]);
        b.push8(0x2014,[1]);
        b.q335ac=VecDeque::from([BtStage111Regs{r0:0x5000,r1:0x1111,r2:0x2222,r3:0x3333}]);
        assert_eq!(bt_stage111_record_dispatch(0x1000,0x2000,0x3000,4,&mut b),0xB04A);
        assert!(b.e.contains(&E::Call("3b04a",0x2000,0x1111,0x2222,0)));
    }

    #[test]
    fn buffer_boundary_gets_raw_state0e_and_is_final() {
        let mut b=B::default();
        b.push8(0x3006,[0]); b.push8(0x100F,[0]);
        b.push8(0x1090,[0x00]);
        b.push8(0x200F,[0]);
        b.push8(0x200E,[0xA5]);
        assert_eq!(bt_stage111_record_dispatch(0x1000,0x2000,0x3000,4,&mut b),0x3D24);
        assert!(b.e.contains(&E::Call("3d24",0x2010,0,0x74,0xA5)));
        assert!(b.e.contains(&E::Write8(0x2001,0)));
    }

    #[test]
    fn bounded_counter_only_increments_below_state14_high3_limit() {
        let mut b=B::default();
        b.push8(0x3006,[0]); b.push8(0x100F,[0]);
        b.push8(0x1090,[0x00]);
        b.push8(0x200F,[0]);
        b.push8(0x200E,[0]);
        b.push8(0x2014,[0x60]);
        b.push8(0x2001,[1,2]);
        let _=bt_stage111_record_dispatch(0x1000,0x2000,0x3000,4,&mut b);
        assert!(b.e.contains(&E::Write8(0x2001,3)));
    }

    #[test]
    fn provenance_constants_are_exact() {
        assert_eq!(STAGE111_CURRENT_BT_RECORD_DISPATCH_ADDR,0x16DBDC);
        assert_eq!(STAGE111_LEGACY_BT_RECORD_DISPATCH_ADDR,0x16AC10);
        assert_eq!(STAGE111_CURRENT_BODY_LEN,326);
        assert_eq!(STAGE111_RECORD_STRIDE,0x19);
        assert_eq!(STAGE111_BT_FIRST_BOUNDARY,0x335AC);
        assert_eq!(STAGE111_BT_RECORD_VALUE_BOUNDARY,0x3A6CC);
        assert_eq!(STAGE111_BT_NOTIFY_BOUNDARY,0x3B04A);
        assert_eq!(STAGE111_BT_BUFFER_BOUNDARY,0x3D24);
    }
}
pub const STAGE112_CURRENT_BT_RECORD_UPDATE_ADDR: u32 = 0x0016_DD22;
pub const STAGE112_LEGACY_BT_RECORD_UPDATE_ADDR: u32 = 0x0016_AD56;
pub const STAGE112_CURRENT_BODY_LEN: u32 = 136;
pub const STAGE112_RECORD_STRIDE: u32 = 0x19;

pub const STAGE112_BT_FIRST_BOUNDARY: u32 = 0x0001_7E2C;
pub const STAGE112_BT_GATE_BOUNDARY: u32 = 0x0003_AF86;
pub const STAGE112_BT_OBJECT_BOUNDARY: u32 = 0x0003_C7C2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage112Regs {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
}

pub trait BtStage112Backend {
    fn read_object_byte(&mut self, object: u32, offset: u32) -> u8;
    fn read_state_byte(&mut self, state: u32, offset: u32) -> u8;
    fn read_state_halfword(&mut self, state: u32, offset: u32) -> u16;
    fn write_state_byte(&mut self, state: u32, offset: u32, value: u8);
    fn write_state_halfword(&mut self, state: u32, offset: u32, value: u16);
    fn write_state_word(&mut self, state: u32, offset: u32, value: u32);

    fn boundary_17e2c(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage112Regs;
    fn boundary_3af86(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage112Regs;
    fn boundary_3c7c2(&mut self, r0:u32,r1:u32,r2:u32,r3:u32)->BtStage112Regs;
}

/// Exact register-state model of current `0x16DD22..0x16DDAA`.
///
/// State byte +1 is snapshotted before the first opaque boundary and selects a
/// 0x19-byte record. The first boundary's R0 remains live through the local
/// gates and becomes both the record word source (`R0 << 1`) and, on some
/// paths, the final function return.
pub fn bt_stage112_record_update<B: BtStage112Backend>(
    object: u32,
    incoming_r1: u32,
    state: u32,
    incoming_r3: u32,
    backend: &mut B,
) -> u32 {
    let object0e = backend.read_object_byte(object, 0x0E);
    let state1_snapshot = backend.read_state_byte(state, 1);

    let mut regs = backend.boundary_17e2c(
        u32::from(object0e),
        incoming_r1,
        state,
        incoming_r3,
    );

    let object98 = backend.read_object_byte(object, 0x98);
    regs.r3 = u32::from((object98 >> 3) & 0x0F);
    if regs.r3 <= 2 {
        return regs.r0;
    }

    let object9a = backend.read_object_byte(object, 0x9A);
    regs.r3 = u32::from(object9a & 0x03).wrapping_sub(1);
    if regs.r3 > 1 {
        return regs.r0;
    }

    let record = state.wrapping_add(
        STAGE112_RECORD_STRIDE.wrapping_mul(u32::from(state1_snapshot))
    );
    regs.r3 = regs.r0.wrapping_shl(1);
    backend.write_state_word(record, 0x35, regs.r3);

    if incoming_r1 == 0 {
        let state14 = backend.read_state_byte(state, 0x14);
        regs.r2 = u32::from(state14);
        regs.r3 = ((regs.r2 >> 5).wrapping_add(1)) & 7;
        let mut next = (state14 & 0x1F) | ((regs.r3 as u8) << 5);
        backend.write_state_byte(state, 0x14, next);
        if regs.r3 > 3 {
            next |= 1;
            backend.write_state_byte(state, 0x14, next);
        }
        return regs.r0;
    }

    regs.r1 = state;
    regs = backend.boundary_3af86(regs.r0, regs.r1, regs.r2, regs.r3);
    if regs.r0 != 0 {
        let object0f = backend.read_object_byte(object, 0x0F);
        regs.r3 = u32::from(object0f);
        backend.write_state_byte(state, 0x12, object0f);

        regs.r0 = object;
        regs = backend.boundary_3c7c2(regs.r0, regs.r1, regs.r2, regs.r3);

        let state4 = backend.read_state_halfword(state, 4);
        backend.write_state_halfword(state, 0x10, state4);

        let state14 = backend.read_state_byte(state, 0x14);
        let zero_result = u32::from(regs.r0 == 0);
        regs.r0 = zero_result;
        backend.write_state_byte(state, 0x14, state14.wrapping_add(0x20));
        backend.write_state_byte(state, 0x13, zero_result as u8);
        backend.write_state_byte(state, 0x0F, 1);
    }

    backend.write_state_byte(state, 0x0E, 1);
    regs.r0
}

#[cfg(test)]
mod stage112_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    #[derive(Clone,Debug,PartialEq,Eq)]
    enum E {
        Read(&'static str,u32,u32),
        Write(&'static str,u32,u32),
        Call(&'static str,u32,u32,u32,u32),
    }

    struct B {
        object0e:u8, object98:u8, object9a:u8, object0f:u8,
        state1:u8, state14:VecDeque<u8>, state4:u16,
        q17e2c:VecDeque<BtStage112Regs>,
        q3af86:VecDeque<BtStage112Regs>,
        q3c7c2:VecDeque<BtStage112Regs>,
        e:Vec<E>,
    }

    impl Default for B {
        fn default()->Self { Self {
            object0e:0x0E, object98:0x18, object9a:1, object0f:0x0F,
            state1:2, state14:VecDeque::from([0]), state4:0x1234,
            q17e2c:VecDeque::from([BtStage112Regs{r0:5,r1:0x11,r2:0x22,r3:0x33}]),
            q3af86:VecDeque::from([BtStage112Regs{r0:1,r1:0x41,r2:0x42,r3:0x43}]),
            q3c7c2:VecDeque::from([BtStage112Regs{r0:7,r1:0x51,r2:0x52,r3:0x53}]),
            e:Vec::new(),
        }}
    }

    impl B {
        fn pop(q:&mut VecDeque<BtStage112Regs>)->BtStage112Regs { q.pop_front().unwrap() }
    }

    impl BtStage112Backend for B {
        fn read_object_byte(&mut self,o:u32,off:u32)->u8 {
            let v=match off {0x0e=>self.object0e,0x98=>self.object98,0x9a=>self.object9a,0x0f=>self.object0f,_=>panic!()};
            self.e.push(E::Read("ob",o.wrapping_add(off),u32::from(v)));v
        }
        fn read_state_byte(&mut self,s:u32,off:u32)->u8 {
            let v=match off {1=>self.state1,0x14=>self.state14.pop_front().unwrap(),_=>panic!()};
            self.e.push(E::Read("sb",s.wrapping_add(off),u32::from(v)));v
        }
        fn read_state_halfword(&mut self,s:u32,off:u32)->u16 {
            assert_eq!(off,4);self.e.push(E::Read("sh",s.wrapping_add(off),u32::from(self.state4)));self.state4
        }
        fn write_state_byte(&mut self,s:u32,off:u32,v:u8){self.e.push(E::Write("sb",s.wrapping_add(off),u32::from(v)))}
        fn write_state_halfword(&mut self,s:u32,off:u32,v:u16){self.e.push(E::Write("sh",s.wrapping_add(off),u32::from(v)))}
        fn write_state_word(&mut self,s:u32,off:u32,v:u32){self.e.push(E::Write("sw",s.wrapping_add(off),v))}
        fn boundary_17e2c(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage112Regs{
            self.e.push(E::Call("17e2c",a,b,c,d));Self::pop(&mut self.q17e2c)
        }
        fn boundary_3af86(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage112Regs{
            self.e.push(E::Call("3af86",a,b,c,d));Self::pop(&mut self.q3af86)
        }
        fn boundary_3c7c2(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage112Regs{
            self.e.push(E::Call("3c7c2",a,b,c,d));Self::pop(&mut self.q3c7c2)
        }
    }

    #[test]
    fn first_boundary_gets_exact_entry_args_and_low_object98_exits_with_live_r0() {
        let mut b=B::default();b.object98=0x10;
        b.q17e2c=VecDeque::from([BtStage112Regs{r0:0xCAFE,r1:2,r2:3,r3:4}]);
        assert_eq!(bt_stage112_record_update(0x1000,0x2000,0x3000,0x4444,&mut b),0xCAFE);
        assert!(b.e.contains(&E::Call("17e2c",0x0E,0x2000,0x3000,0x4444)));
        assert!(!b.e.iter().any(|e|matches!(e,E::Write("sw",..))));
    }

    #[test]
    fn invalid_low2_gate_exits_before_record_write() {
        let mut b=B::default();b.object9a=0;
        assert_eq!(bt_stage112_record_update(0x1000,1,0x3000,4,&mut b),5);
        assert!(!b.e.iter().any(|e|matches!(e,E::Write("sw",..)|E::Call("3af86",..))));
    }

    #[test]
    fn record_uses_preboundary_state1_snapshot_and_doubled_first_return() {
        let mut b=B::default();b.state1=3;b.object98=0x18;b.object9a=2;
        let _=bt_stage112_record_update(0x1000,0,0x3000,4,&mut b);
        assert!(b.e.contains(&E::Write("sw",0x3000+3*0x19+0x35,10)));
    }

    #[test]
    fn zero_incoming_r1_rotates_high3_field_without_opaque_followups() {
        let mut b=B::default();b.state14=VecDeque::from([0x65]);
        assert_eq!(bt_stage112_record_update(0x1000,0,0x3000,4,&mut b),5);
        assert!(b.e.contains(&E::Write("sb",0x3014,0x85)));
        assert!(b.e.contains(&E::Write("sb",0x3014,0x85)));
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("3af86",..)|E::Call("3c7c2",..))));
    }

    #[test]
    fn zero_incoming_r1_wraps_high3_field_and_preserves_low5() {
        let mut b=B::default();b.state14=VecDeque::from([0xE5]);
        let _=bt_stage112_record_update(0x1000,0,0x3000,4,&mut b);
        assert!(b.e.contains(&E::Write("sb",0x3014,0x05)));
    }

    #[test]
    fn gate_boundary_gets_live_r2_and_doubled_first_return_and_zero_sets_only_byte0e() {
        let mut b=B::default();
        b.q17e2c=VecDeque::from([BtStage112Regs{r0:7,r1:8,r2:0xCAFE,r3:0xDEAD}]);
        b.q3af86=VecDeque::from([BtStage112Regs{r0:0,r1:0x41,r2:0x42,r3:0x43}]);
        assert_eq!(bt_stage112_record_update(0x1000,1,0x3000,4,&mut b),0);
        assert!(b.e.contains(&E::Call("3af86",7,0x3000,0xCAFE,14)));
        assert!(b.e.contains(&E::Write("sb",0x300E,1)));
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("3c7c2",..))));
    }

    #[test]
    fn object_boundary_chain_copies_state_and_returns_zero_boolean_for_nonzero_result() {
        let mut b=B::default();b.object0f=0xA5;b.state4=0xBEEF;b.state14=VecDeque::from([0xF1]);
        b.q3af86=VecDeque::from([BtStage112Regs{r0:2,r1:0xAAAA,r2:0xBBBB,r3:0xCCCC}]);
        b.q3c7c2=VecDeque::from([BtStage112Regs{r0:9,r1:0x51,r2:0x52,r3:0x53}]);
        assert_eq!(bt_stage112_record_update(0x1000,1,0x3000,4,&mut b),0);
        assert!(b.e.contains(&E::Write("sb",0x3012,0xA5)));
        assert!(b.e.contains(&E::Call("3c7c2",0x1000,0xAAAA,0xBBBB,0xA5)));
        assert!(b.e.contains(&E::Write("sh",0x3010,0xBEEF)));
        assert!(b.e.contains(&E::Write("sb",0x3014,0x11)));
        assert!(b.e.contains(&E::Write("sb",0x3013,0)));
        assert!(b.e.contains(&E::Write("sb",0x300F,1)));
        assert!(b.e.contains(&E::Write("sb",0x300E,1)));
    }

    #[test]
    fn zero_object_boundary_return_becomes_literal_one_and_sets_state13() {
        let mut b=B::default();b.state14=VecDeque::from([0x20]);
        b.q3af86=VecDeque::from([BtStage112Regs{r0:2,r1:3,r2:4,r3:5}]);
        b.q3c7c2=VecDeque::from([BtStage112Regs{r0:0,r1:6,r2:7,r3:8}]);
        assert_eq!(bt_stage112_record_update(0x1000,1,0x3000,4,&mut b),1);
        assert!(b.e.contains(&E::Write("sb",0x3013,1)));
    }

    #[test]
    fn provenance_constants_are_exact() {
        assert_eq!(STAGE112_CURRENT_BT_RECORD_UPDATE_ADDR,0x16DD22);
        assert_eq!(STAGE112_LEGACY_BT_RECORD_UPDATE_ADDR,0x16AD56);
        assert_eq!(STAGE112_CURRENT_BODY_LEN,136);
        assert_eq!(STAGE112_BT_FIRST_BOUNDARY,0x17E2C);
        assert_eq!(STAGE112_BT_GATE_BOUNDARY,0x3AF86);
        assert_eq!(STAGE112_BT_OBJECT_BOUNDARY,0x3C7C2);
    }
}
pub const STAGE113_CURRENT_BT_RESOURCE_DISPATCH_ADDR: u32 = 0x0016_DDAC;
pub const STAGE113_LEGACY_BT_RESOURCE_DISPATCH_ADDR: u32 = 0x0016_ADE0;
pub const STAGE113_CURRENT_BODY_LEN: u32 = 236;
pub const STAGE113_GLOBAL_FLAG_BASE_ADDR: u32 = 0x0020_8338;

pub const STAGE113_BT_RESOLVE_BOUNDARY: u32 = 0x0001_EE18;
pub const STAGE113_BT_SELECT_BOUNDARY: u32 = 0x0001_F270;
pub const STAGE113_BT_PROBE_BOUNDARY: u32 = 0x0000_0780;
pub const STAGE113_BT_DESTROY_BOUNDARY: u32 = 0x000B_0460;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage113Regs {
    pub r0:u32, pub r1:u32, pub r2:u32, pub r3:u32,
}

pub trait BtStage113Backend {
    fn read_input_word(&mut self, base:u32, offset:u32)->u32;
    fn read_object_word(&mut self, object:u32, offset:u32)->u32;
    fn write_object_word(&mut self, object:u32, offset:u32, value:u32);
    fn read_object_byte(&mut self, object:u32, offset:u32)->u8;
    fn read_indirect_byte(&mut self, ptr:u32, offset:u32)->u8;
    fn read_global_flag_byte13(&mut self)->u8;

    fn boundary_1ee18(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage113Regs;
    fn boundary_1f270(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage113Regs;
    fn boundary_780(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage113Regs;
    fn boundary_b0460(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage113Regs;
    fn tail_780(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage113Regs;
}

/// Exact register-state model of current `0x16DDAC..0x16DE98`.
///
/// The model preserves the table-branch selector, every conditional/unconditional
/// resource destruction, caller-volatile R1-R3 across opaque boundaries, the
/// global flag gate, and the final pop+tail argument shape.
pub fn bt_stage113_resource_dispatch<B: BtStage113Backend>(
    input:u32,
    incoming_r1:u32,
    incoming_r2:u32,
    _incoming_r3:u32,
    backend:&mut B,
)->u32{
    let holder = backend.read_input_word(input, 4);
    let handle = backend.read_input_word(holder, 0);
    let mut regs = backend.boundary_1ee18(handle, incoming_r1, incoming_r2, holder);
    let object = regs.r0;

    let word10 = backend.read_object_word(object, 0x10);
    regs.r3 = word10;
    if word10 != 0 {
        return regs.r0;
    }

    let selector_byte = backend.read_object_byte(object, 0x0B);
    let selector = (selector_byte >> 2) & 0x0F;
    regs.r3 = u32::from(selector);

    let mut r5:u32;
    match selector {
        0 => {
            r5 = backend.read_object_word(object, 0x20);
            if r5 != 0 {
                regs.r0 = object;
                regs = backend.boundary_1f270(regs.r0,regs.r1,regs.r2,regs.r3);
                r5 = regs.r0;
            }
        }
        1 => {
            r5 = backend.read_object_word(object, 0x14);
            if r5 != 0 {
                regs.r0 = object;
                regs = backend.boundary_1f270(regs.r0,regs.r1,regs.r2,regs.r3);
                r5 = regs.r0;
            }
        }
        2 => {
            regs.r0 = 1;
            regs = backend.boundary_780(regs.r0,regs.r1,regs.r2,regs.r3);
            let saved_probe_r0 = regs.r0;

            let mut cursor = object;
            let end = object.wrapping_add(0x24);
            for _ in 0..3 {
                let word = backend.read_object_word(cursor, 0x14);
                regs.r0 = word;
                if word != 0 {
                    regs = backend.boundary_b0460(regs.r0,regs.r1,regs.r2,regs.r3);
                    backend.write_object_word(cursor,0x14,0);
                }
                cursor = cursor.wrapping_add(0x0C);
            }
            r5 = end;

            let word = backend.read_object_word(object,0x10);
            regs.r0 = word;
            if word != 0 {
                regs = backend.boundary_b0460(regs.r0,regs.r1,regs.r2,regs.r3);
                backend.write_object_word(object,0x10,0);
            }

            regs.r0 = saved_probe_r0;
            regs = backend.boundary_780(regs.r0,regs.r1,regs.r2,regs.r3);
        }
        3 => {
            r5 = backend.read_object_word(object,0x2C);
            if r5 != 0 {
                regs.r0 = object;
                regs = backend.boundary_1f270(regs.r0,regs.r1,regs.r2,regs.r3);
                r5 = regs.r0;

                regs.r0 = 1;
                regs = backend.boundary_780(regs.r0,regs.r1,regs.r2,regs.r3);
                let saved_probe_r0 = regs.r0;

                if r5 != 1 {
                    let word20 = backend.read_object_word(object,0x20);
                    regs.r0 = word20;
                    regs = backend.boundary_b0460(regs.r0,regs.r1,regs.r2,regs.r3);
                    backend.write_object_word(object,0x20,0);

                    let word2c = backend.read_object_word(object,0x2C);
                    regs.r0 = word2c;
                    regs = backend.boundary_b0460(regs.r0,regs.r1,regs.r2,regs.r3);
                    backend.write_object_word(object,0x2C,0);
                }

                regs.r0 = saved_probe_r0;
                regs = backend.boundary_780(regs.r0,regs.r1,regs.r2,regs.r3);
            }
        }
        _ => { r5 = 0; }
    }

    let word14 = backend.read_object_word(object,0x14);
    regs.r3 = word14;
    if word14 != 0 {
        let byte2 = backend.read_indirect_byte(word14,2);
        regs.r3 = u32::from(byte2);
        regs.r2 = regs.r3 << 30;
        if regs.r2 == 0 {
            let flag = backend.read_global_flag_byte13();
            regs.r3 = u32::from(flag) << 28;
            if flag & 0x08 == 0 {
                return regs.r0;
            }
        }
    }

    if r5 != 1 {
        return regs.r0;
    }

    regs.r0 = r5;
    regs = backend.boundary_780(regs.r0,regs.r1,regs.r2,regs.r3);
    let saved_final_probe_r0 = regs.r0;

    let word20 = backend.read_object_word(object,0x20);
    regs.r0 = word20;
    let old14 = backend.read_object_word(object,0x14);
    regs.r3 = old14;
    backend.write_object_word(object,0x10,old14);
    backend.write_object_word(object,0x14,0);

    if word20 != 0 {
        regs = backend.boundary_b0460(regs.r0,regs.r1,regs.r2,regs.r3);
        backend.write_object_word(object,0x20,0);
    }

    let word2c = backend.read_object_word(object,0x2C);
    regs.r0 = word2c;
    if word2c != 0 {
        regs = backend.boundary_b0460(regs.r0,regs.r1,regs.r2,regs.r3);
        backend.write_object_word(object,0x2C,0);
    }

    regs.r0 = saved_final_probe_r0;
    backend.tail_780(regs.r0,regs.r1,regs.r2,regs.r3).r0
}

#[cfg(test)]
mod stage113_tests {
    use super::*;
    use std::collections::{BTreeMap,VecDeque};
    use std::vec::Vec;

    #[derive(Clone,Debug,PartialEq,Eq)]
    enum E { ReadW(u32,u32),ReadB(u32,u8),WriteW(u32,u32),Call(&'static str,u32,u32,u32,u32) }

    struct B{
        words:BTreeMap<u32,VecDeque<u32>>, bytes:BTreeMap<u32,VecDeque<u8>>, flag:u8,
        qresolve:VecDeque<BtStage113Regs>, qselect:VecDeque<BtStage113Regs>,
        qprobe:VecDeque<BtStage113Regs>, qdestroy:VecDeque<BtStage113Regs>, qtail:VecDeque<BtStage113Regs>,
        e:Vec<E>,
    }
    impl Default for B{
        fn default()->Self{
            let mut b=Self{
                words:BTreeMap::new(),bytes:BTreeMap::new(),flag:0,
                qresolve:VecDeque::from([BtStage113Regs{r0:0x2000,r1:0x11,r2:0x22,r3:0x33}]),
                qselect:VecDeque::from([BtStage113Regs{r0:2,r1:0x31,r2:0x32,r3:0x33}]),
                qprobe:VecDeque::from([BtStage113Regs{r0:0x700,r1:0x41,r2:0x42,r3:0x43}]),
                qdestroy:VecDeque::new(),
                qtail:VecDeque::from([BtStage113Regs{r0:0x780,..Default::default()}]),e:Vec::new(),
            };
            b.pushw(0x1004,[0x1100]);b.pushw(0x1100,[0x1200]);
            b.pushw(0x2010,[0]);b.pushb(0x200B,[0]);
            b.pushw(0x2020,[0]);b.pushw(0x2014,[0]);
            b
        }
    }
    impl B{
        fn pushw<const N:usize>(&mut self,a:u32,v:[u32;N]){self.words.insert(a,VecDeque::from(v));}
        fn pushb<const N:usize>(&mut self,a:u32,v:[u8;N]){self.bytes.insert(a,VecDeque::from(v));}
        fn pop(q:&mut VecDeque<BtStage113Regs>)->BtStage113Regs{q.pop_front().unwrap()}
    }
    impl BtStage113Backend for B{
        fn read_input_word(&mut self,b:u32,o:u32)->u32{let a=b.wrapping_add(o);let v=self.words.get_mut(&a).unwrap().pop_front().unwrap();self.e.push(E::ReadW(a,v));v}
        fn read_object_word(&mut self,b:u32,o:u32)->u32{let a=b.wrapping_add(o);let v=self.words.get_mut(&a).unwrap().pop_front().unwrap();self.e.push(E::ReadW(a,v));v}
        fn write_object_word(&mut self,b:u32,o:u32,v:u32){let a=b.wrapping_add(o);self.e.push(E::WriteW(a,v))}
        fn read_object_byte(&mut self,b:u32,o:u32)->u8{let a=b.wrapping_add(o);let v=self.bytes.get_mut(&a).unwrap().pop_front().unwrap();self.e.push(E::ReadB(a,v));v}
        fn read_indirect_byte(&mut self,p:u32,o:u32)->u8{let a=p.wrapping_add(o);let v=self.bytes.get_mut(&a).unwrap().pop_front().unwrap();self.e.push(E::ReadB(a,v));v}
        fn read_global_flag_byte13(&mut self)->u8{let a=STAGE113_GLOBAL_FLAG_BASE_ADDR+0x13;self.e.push(E::ReadB(a,self.flag));self.flag}
        fn boundary_1ee18(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage113Regs{self.e.push(E::Call("1ee18",a,b,c,d));Self::pop(&mut self.qresolve)}
        fn boundary_1f270(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage113Regs{self.e.push(E::Call("1f270",a,b,c,d));Self::pop(&mut self.qselect)}
        fn boundary_780(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage113Regs{self.e.push(E::Call("780",a,b,c,d));Self::pop(&mut self.qprobe)}
        fn boundary_b0460(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage113Regs{self.e.push(E::Call("b0460",a,b,c,d));Self::pop(&mut self.qdestroy)}
        fn tail_780(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage113Regs{self.e.push(E::Call("tail780",a,b,c,d));Self::pop(&mut self.qtail)}
    }

    #[test] fn first_boundary_args_and_word10_early_return_are_exact(){
        let mut b=B::default();b.words.insert(0x2010,VecDeque::from([9]));
        b.qresolve=VecDeque::from([BtStage113Regs{r0:0x2000,r1:2,r2:3,r3:4}]);
        assert_eq!(bt_stage113_resource_dispatch(0x1000,0xAA,0xBB,0xCC,&mut b),0x2000);
        assert!(b.e.contains(&E::Call("1ee18",0x1200,0xAA,0xBB,0x1100)));
    }

    #[test] fn selector0_calls_select_with_selector_in_r3_and_returns_its_r0_when_not_one(){
        let mut b=B::default();b.words.insert(0x2020,VecDeque::from([0x99]));b.words.insert(0x2014,VecDeque::from([0]));
        b.qselect=VecDeque::from([BtStage113Regs{r0:2,r1:3,r2:4,r3:5}]);
        assert_eq!(bt_stage113_resource_dispatch(0x1000,1,2,3,&mut b),2);
        assert!(b.e.contains(&E::Call("1f270",0x2000,0x11,0x22,0)));
    }

    #[test] fn selector2_sweeps_three_slots_and_word10_then_calls_probe_again(){
        let mut b=B::default();b.bytes.insert(0x200B,VecDeque::from([8]));
        b.words.insert(0x2014,VecDeque::from([0xA,0]));b.words.insert(0x2020,VecDeque::from([0]));b.words.insert(0x202C,VecDeque::from([0xC]));
        b.words.insert(0x2010,VecDeque::from([0,0xD]));
        b.qprobe=VecDeque::from([
            BtStage113Regs{r0:0x700,r1:1,r2:2,r3:3},
            BtStage113Regs{r0:0x701,r1:4,r2:5,r3:6},
        ]);
        b.qdestroy=VecDeque::from([
            BtStage113Regs{r0:1,r1:0x11,r2:0x12,r3:0x13},
            BtStage113Regs{r0:2,r1:0x21,r2:0x22,r3:0x23},
            BtStage113Regs{r0:3,r1:0x31,r2:0x32,r3:0x33},
        ]);
        assert_eq!(bt_stage113_resource_dispatch(0x1000,1,2,3,&mut b),0x701);
        assert!(b.e.contains(&E::WriteW(0x2014,0)));
        assert!(!b.e.contains(&E::WriteW(0x2020,0)));
        assert!(b.e.contains(&E::WriteW(0x202C,0)));
        assert!(b.e.contains(&E::WriteW(0x2010,0)));
        assert!(b.e.contains(&E::Call("780",0x700,0x31,0x32,0x33)));
    }

    #[test] fn selector3_nonone_destroys_word20_and_word2c_even_when_zero(){
        let mut b=B::default();b.bytes.insert(0x200B,VecDeque::from([12]));
        b.words.insert(0x202C,VecDeque::from([1,0]));b.words.insert(0x2020,VecDeque::from([0]));b.words.insert(0x2014,VecDeque::from([0]));
        b.qselect=VecDeque::from([BtStage113Regs{r0:2,r1:3,r2:4,r3:5}]);
        b.qprobe=VecDeque::from([
            BtStage113Regs{r0:0x700,r1:0x41,r2:0x42,r3:0x43},
            BtStage113Regs{r0:0x701,r1:0x51,r2:0x52,r3:0x53},
        ]);
        b.qdestroy=VecDeque::from([
            BtStage113Regs{r0:0xD1,r1:0x61,r2:0x62,r3:0x63},
            BtStage113Regs{r0:0xD2,r1:0x71,r2:0x72,r3:0x73},
        ]);
        let _=bt_stage113_resource_dispatch(0x1000,1,2,3,&mut b);
        assert!(b.e.contains(&E::Call("b0460",0,0x41,0x42,0x43)));
        assert!(b.e.contains(&E::Call("b0460",0,0x61,0x62,0x63)));
    }

    #[test] fn low2_clear_and_global_bit3_clear_exits_before_r5_one_transition(){
        let mut b=B::default();b.bytes.insert(0x200B,VecDeque::from([0]));
        b.words.insert(0x2020,VecDeque::from([1]));b.words.insert(0x2014,VecDeque::from([0x3000]));
        b.bytes.insert(0x3002,VecDeque::from([0]));b.flag=0;
        b.qselect=VecDeque::from([BtStage113Regs{r0:1,r1:2,r2:3,r3:4}]);
        assert_eq!(bt_stage113_resource_dispatch(0x1000,1,2,3,&mut b),1);
        assert!(!b.e.iter().any(|e|matches!(e,E::Call("tail780",..))));
    }

    #[test] fn r5_one_transition_moves_word14_and_tails_with_live_destroy_regs(){
        let mut b=B::default();b.bytes.insert(0x200B,VecDeque::from([0]));
        b.words.insert(0x2020,VecDeque::from([1,0xA0]));b.words.insert(0x2014,VecDeque::from([0,0xB0]));
        b.words.insert(0x202C,VecDeque::from([0xC0]));
        b.qselect=VecDeque::from([BtStage113Regs{r0:1,r1:0x31,r2:0x32,r3:0x33}]);
        b.qprobe=VecDeque::from([BtStage113Regs{r0:0x700,r1:0x41,r2:0x42,r3:0x43}]);
        b.qdestroy=VecDeque::from([
            BtStage113Regs{r0:0xD1,r1:0x51,r2:0x52,r3:0x53},
            BtStage113Regs{r0:0xD2,r1:0x61,r2:0x62,r3:0x63},
        ]);
        assert_eq!(bt_stage113_resource_dispatch(0x1000,1,2,3,&mut b),0x780);
        assert!(b.e.contains(&E::WriteW(0x2010,0xB0)));
        assert!(b.e.contains(&E::WriteW(0x2014,0)));
        assert!(b.e.contains(&E::Call("tail780",0x700,0x61,0x62,0x63)));
    }

    #[test] fn provenance_constants_are_exact(){
        assert_eq!(STAGE113_CURRENT_BT_RESOURCE_DISPATCH_ADDR,0x16DDAC);
        assert_eq!(STAGE113_LEGACY_BT_RESOURCE_DISPATCH_ADDR,0x16ADE0);
        assert_eq!(STAGE113_CURRENT_BODY_LEN,236);
        assert_eq!(STAGE113_GLOBAL_FLAG_BASE_ADDR,0x208338);
        assert_eq!(STAGE113_BT_RESOLVE_BOUNDARY,0x1EE18);
        assert_eq!(STAGE113_BT_SELECT_BOUNDARY,0x1F270);
        assert_eq!(STAGE113_BT_PROBE_BOUNDARY,0x780);
        assert_eq!(STAGE113_BT_DESTROY_BOUNDARY,0xB0460);
    }
}
pub const STAGE114_CURRENT_BT_SLOT_REWRITE_ADDR: u32 = 0x0016_DE9C;
pub const STAGE114_LEGACY_BT_SLOT_REWRITE_ADDR: u32 = 0x0016_AED0;
pub const STAGE114_CURRENT_BODY_LEN: u32 = 116;

pub const STAGE114_BT_RESOLVE_BOUNDARY: u32 = 0x0001_EE18;
pub const STAGE114_BT_DESTROY_BOUNDARY: u32 = 0x000B_0460;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage114Regs {
    pub r0:u32, pub r1:u32, pub r2:u32, pub r3:u32,
}

pub trait BtStage114Backend {
    fn read_input_byte(&mut self, input:u32, offset:u32)->u8;
    fn write_input_byte(&mut self, input:u32, offset:u32, value:u8);

    fn read_slot_word(&mut self, slot:u32)->u32;
    fn write_slot_word(&mut self, slot:u32, value:u32);

    fn read_object_word(&mut self, object:u32, offset:u32)->u32;
    fn write_object_word(&mut self, object:u32, offset:u32, value:u32);
    fn read_object_byte(&mut self, object:u32, offset:u32)->u8;
    fn write_object_byte(&mut self, object:u32, offset:u32, value:u8);

    fn read_indirect_byte(&mut self, ptr:u32, offset:u32)->u8;
    fn read_indirect_halfword(&mut self, ptr:u32, offset:u32)->u16;

    fn read_state_byte(&mut self, state:u32, offset:u32)->u8;
    fn write_state_byte(&mut self, state:u32, offset:u32, value:u8);
    fn read_state_halfword(&mut self, state:u32, offset:u32)->u16;
    fn write_state_halfword(&mut self, state:u32, offset:u32, value:u16);

    fn boundary_1ee18(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage114Regs;
    fn boundary_b0460(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage114Regs;
}

/// Exact register-state model of current `0x16DE9C..0x16DF10`.
///
/// The resolver receives input byte +0x14 with incoming R1/R2/R3. Its full
/// caller-volatile return reaches the unconditional destroy of the old slot
/// word. The destroy return's R0 remains live to the function return.
pub fn bt_stage114_slot_rewrite<B: BtStage114Backend>(
    input:u32,
    slot:u32,
    state:u32,
    incoming_r3:u32,
    backend:&mut B,
)->u32{
    let input14=backend.read_input_byte(input,0x14);
    let mut regs=backend.boundary_1ee18(u32::from(input14),slot,state,incoming_r3);
    let object=regs.r0;

    regs.r0=backend.read_slot_word(slot);
    regs=backend.boundary_b0460(regs.r0,regs.r1,regs.r2,regs.r3);

    let descriptor=backend.read_object_word(object,0x10);
    backend.write_slot_word(slot,descriptor);

    regs.r3=2;
    backend.write_state_byte(state,0x1D,2);

    let object0b=backend.read_object_byte(object,0x0B);
    regs.r3=u32::from(object0b & 0xC0);
    regs.r2=descriptor;

    if regs.r3==0x40 {
        let b=backend.read_indirect_byte(descriptor,2);
        regs.r3=u32::from(b)>>3;
        let old=backend.read_state_halfword(state,0x1A);
        regs.r2=u32::from(old);
        let field=(regs.r3 & 0x3FF)<<3;
        regs.r2=(regs.r2 & !(0x3FF<<3)) | field;
        backend.write_state_halfword(state,0x1A,regs.r2 as u16);
    } else if regs.r3==0x80 {
        let h=backend.read_indirect_halfword(descriptor,2);
        regs.r3=u32::from(h)>>3;
        let old=backend.read_state_halfword(state,0x1A);
        regs.r2=u32::from(old);
        let field=(regs.r3 & 0x3FF)<<3;
        regs.r2=(regs.r2 & !(0x3FF<<3)) | field;
        backend.write_state_halfword(state,0x1A,regs.r2 as u16);
    }

    let descriptor2=backend.read_object_word(object,0x10);
    regs.r2=backend.read_indirect_byte(descriptor2,2) as u32;
    let old1a=backend.read_state_byte(state,0x1A);
    regs.r3=u32::from(old1a);
    regs.r3=(regs.r3 & !0x03) | (regs.r2 & 0x03);
    backend.write_state_byte(state,0x1A,regs.r3 as u8);

    let descriptor3=backend.read_object_word(object,0x10);
    let source=backend.read_indirect_byte(descriptor3,2);
    regs.r3=u32::from(source)>>2;
    regs.r2=u32::from(backend.read_state_byte(state,0x1A));
    regs.r2=(regs.r2 & !(1<<2)) | ((regs.r3 & 1)<<2);
    backend.write_state_byte(state,0x1A,regs.r2 as u8);

    let input9=backend.read_input_byte(input,9);
    regs.r3=u32::from(input9 | 1);
    backend.write_input_byte(input,9,regs.r3 as u8);

    let fresh0b=backend.read_object_byte(object,0x0B);
    regs.r3=u32::from((fresh0b & 0xFC) | 0x3C);
    backend.write_object_byte(object,0x0B,regs.r3 as u8);

    regs.r3=0;
    backend.write_object_word(object,0x10,0);

    regs.r0
}

#[cfg(test)]
mod stage114_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    #[derive(Clone,Debug,PartialEq,Eq)]
    enum E {
        Read(&'static str,u32,u32),
        Write(&'static str,u32,u32),
        Call(&'static str,u32,u32,u32,u32),
    }

    struct B {
        input14:u8,input9:u8,slot_word:u32,
        object10:VecDeque<u32>,object0b:VecDeque<u8>,
        indirect_byte:u8,indirect_half:u16,
        state1a_byte:VecDeque<u8>,state1a_half:u16,
        qresolve:VecDeque<BtStage114Regs>,qdestroy:VecDeque<BtStage114Regs>,
        e:Vec<E>,
    }
    impl Default for B {
        fn default()->Self{Self{
            input14:0x14,input9:0x10,slot_word:0xAAAA,
            object10:VecDeque::from([0x5000,0x5000,0x5000]),
            object0b:VecDeque::from([0,0xFF]),
            indirect_byte:0xA5,indirect_half:0x3456,
            state1a_byte:VecDeque::from([0xF0,0xF3]),
            state1a_half:0xF007,
            qresolve:VecDeque::from([BtStage114Regs{r0:0x2000,r1:0x11,r2:0x22,r3:0x33}]),
            qdestroy:VecDeque::from([BtStage114Regs{r0:0xD00D,r1:0x41,r2:0x42,r3:0x43}]),
            e:Vec::new(),
        }}
    }
    impl B{fn pop(q:&mut VecDeque<BtStage114Regs>)->BtStage114Regs{q.pop_front().unwrap()}}
    impl BtStage114Backend for B {
        fn read_input_byte(&mut self,o:u32,off:u32)->u8{let v=if off==0x14{self.input14}else{assert_eq!(off,9);self.input9};self.e.push(E::Read("ib",o+off,u32::from(v)));v}
        fn write_input_byte(&mut self,o:u32,off:u32,v:u8){self.e.push(E::Write("ib",o+off,u32::from(v)))}
        fn read_slot_word(&mut self,s:u32)->u32{self.e.push(E::Read("slot",s,self.slot_word));self.slot_word}
        fn write_slot_word(&mut self,s:u32,v:u32){self.e.push(E::Write("slot",s,v))}
        fn read_object_word(&mut self,o:u32,off:u32)->u32{assert_eq!(off,0x10);let v=self.object10.pop_front().unwrap();self.e.push(E::Read("ow",o+off,v));v}
        fn write_object_word(&mut self,o:u32,off:u32,v:u32){self.e.push(E::Write("ow",o+off,v))}
        fn read_object_byte(&mut self,o:u32,off:u32)->u8{assert_eq!(off,0x0B);let v=self.object0b.pop_front().unwrap();self.e.push(E::Read("ob",o+off,u32::from(v)));v}
        fn write_object_byte(&mut self,o:u32,off:u32,v:u8){self.e.push(E::Write("ob",o+off,u32::from(v)))}
        fn read_indirect_byte(&mut self,p:u32,off:u32)->u8{assert_eq!(off,2);self.e.push(E::Read("ib2",p+off,u32::from(self.indirect_byte)));self.indirect_byte}
        fn read_indirect_halfword(&mut self,p:u32,off:u32)->u16{assert_eq!(off,2);self.e.push(E::Read("ih2",p+off,u32::from(self.indirect_half)));self.indirect_half}
        fn read_state_byte(&mut self,s:u32,off:u32)->u8{assert_eq!(off,0x1A);let v=self.state1a_byte.pop_front().unwrap();self.e.push(E::Read("sb",s+off,u32::from(v)));v}
        fn write_state_byte(&mut self,s:u32,off:u32,v:u8){self.e.push(E::Write("sb",s+off,u32::from(v)))}
        fn read_state_halfword(&mut self,s:u32,off:u32)->u16{assert_eq!(off,0x1A);self.e.push(E::Read("sh",s+off,u32::from(self.state1a_half)));self.state1a_half}
        fn write_state_halfword(&mut self,s:u32,off:u32,v:u16){self.e.push(E::Write("sh",s+off,u32::from(v)))}
        fn boundary_1ee18(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage114Regs{self.e.push(E::Call("1ee18",a,b,c,d));Self::pop(&mut self.qresolve)}
        fn boundary_b0460(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage114Regs{self.e.push(E::Call("b0460",a,b,c,d));Self::pop(&mut self.qdestroy)}
    }

    #[test]
    fn resolver_and_unconditional_destroy_preserve_exact_live_args_and_final_r0(){
        let mut b=B::default();
        assert_eq!(bt_stage114_slot_rewrite(0x1000,0x3000,0x4000,0x44,&mut b),0xD00D);
        assert!(b.e.contains(&E::Call("1ee18",0x14,0x3000,0x4000,0x44)));
        assert!(b.e.contains(&E::Call("b0460",0xAAAA,0x11,0x22,0x33)));
        assert!(b.e.contains(&E::Write("slot",0x3000,0x5000)));
    }

    #[test]
    fn selector_0x40_inserts_ten_bit_byte_derived_field(){
        let mut b=B::default();b.object0b=VecDeque::from([0x40,0]);
        let _=bt_stage114_slot_rewrite(0x1000,0x3000,0x4000,0,&mut b);
        let src=(u32::from(0xA5u8)>>3)&0x3FF;
        let expected=((0xF007u32 & !(0x3FF<<3)) | (src<<3)) as u16;
        assert!(b.e.contains(&E::Write("sh",0x401A,u32::from(expected))));
    }

    #[test]
    fn selector_0x80_inserts_ten_bit_halfword_derived_field(){
        let mut b=B::default();b.object0b=VecDeque::from([0x80,0]);
        let _=bt_stage114_slot_rewrite(0x1000,0x3000,0x4000,0,&mut b);
        let src=(u32::from(0x3456u16)>>3)&0x3FF;
        let expected=((0xF007u32 & !(0x3FF<<3)) | (src<<3)) as u16;
        assert!(b.e.contains(&E::Write("sh",0x401A,u32::from(expected))));
    }

    #[test]
    fn other_selector_skips_halfword_field_but_still_rewrites_low_three_bits(){
        let mut b=B::default();b.object0b=VecDeque::from([0xC0,0]);
        let _=bt_stage114_slot_rewrite(0x1000,0x3000,0x4000,0,&mut b);
        assert!(!b.e.iter().any(|e|matches!(e,E::Write("sh",..))));
        assert!(b.e.contains(&E::Write("sb",0x401A,0xF1)));
        assert!(b.e.contains(&E::Write("sb",0x401A,0xF7)));
    }

    #[test]
    fn final_input_and_object_writes_are_exact(){
        let mut b=B::default();b.object0b=VecDeque::from([0,0xC3]);b.input9=0xA4;
        let _=bt_stage114_slot_rewrite(0x1000,0x3000,0x4000,0,&mut b);
        assert!(b.e.contains(&E::Write("ib",0x1009,0xA5)));
        assert!(b.e.contains(&E::Write("ob",0x200B,0xFC)));
        assert!(b.e.contains(&E::Write("ow",0x2010,0)));
        assert!(b.e.contains(&E::Write("sb",0x401D,2)));
    }

    #[test]
    fn provenance_constants_are_exact(){
        assert_eq!(STAGE114_CURRENT_BT_SLOT_REWRITE_ADDR,0x16DE9C);
        assert_eq!(STAGE114_LEGACY_BT_SLOT_REWRITE_ADDR,0x16AED0);
        assert_eq!(STAGE114_CURRENT_BODY_LEN,116);
        assert_eq!(STAGE114_BT_RESOLVE_BOUNDARY,0x1EE18);
        assert_eq!(STAGE114_BT_DESTROY_BOUNDARY,0xB0460);
    }
}
pub const STAGE115_CURRENT_BT_SELECTOR_DISPATCH_ADDR: u32 = 0x0016_DF10;
pub const STAGE115_LEGACY_BT_SELECTOR_DISPATCH_ADDR: u32 = 0x0016_AF44;
pub const STAGE115_CURRENT_BODY_LEN: u32 = 450;
pub const STAGE115_GLOBAL_WORD_ADDR: u32 = 0x0031_89DC;
pub const STAGE115_CURRENT_STAGE114_TAIL_ADDR: u32 = 0x0016_DE9C;

pub const STAGE115_BT_FIRST_BOUNDARY: u32 = 0x0003_35AC;
pub const STAGE115_BT_RESOLVE_BOUNDARY: u32 = 0x0001_EE18;
pub const STAGE115_BT_GATE_BOUNDARY: u32 = 0x0001_F3BC;
pub const STAGE115_BT_DESTROY_TAIL: u32 = 0x000B_0460;
pub const STAGE115_BT_FINAL_TAIL: u32 = 0x0001_F3E0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BtStage115Regs {
    pub r0:u32, pub r1:u32, pub r2:u32, pub r3:u32,
}

pub trait BtStage115Backend {
    fn read_byte(&mut self, addr:u32)->u8;
    fn read_halfword(&mut self, addr:u32)->u16;
    fn read_word(&mut self, addr:u32)->u32;
    fn write_byte(&mut self, addr:u32, value:u8);
    fn write_halfword(&mut self, addr:u32, value:u16);
    fn write_word(&mut self, addr:u32, value:u32);

    fn read_global_word_3189dc(&mut self)->u32;

    fn boundary_335ac(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage115Regs;
    fn boundary_1ee18(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage115Regs;
    fn boundary_1f3bc(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage115Regs;

    fn tail_stage114(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage115Regs;
    fn tail_b0460(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage115Regs;
    fn tail_1f3e0(&mut self,r0:u32,r1:u32,r2:u32,r3:u32)->BtStage115Regs;
}

fn bfi_u32(dst:u32, src:u32, lsb:u32, width:u32)->u32 {
    let low_mask = if width == 32 { u32::MAX } else { (1u32 << width) - 1 };
    let mask = low_mask << lsb;
    (dst & !mask) | ((src & low_mask) << lsb)
}

fn selector4(byte:u8)->u32 {
    (u32::from(byte) >> 2) & 0x0F
}

/// Exact register-state model of current `0x16DF10..0x16E0D2`.
///
/// This preserves the two entry opaque calls, the special state-byte shortcut,
/// both true reads of global dword `0x3189DC`, the 16-entry TBB selector map,
/// every path-dependent record initialization, and all three tail shapes.
/// Direct tail to current Stage-114 entry `0x16DE9C` remains an explicit backend
/// edge so Stage 115 does not assume higher-level semantics from its neighbor.
pub fn bt_stage115_selector_dispatch<B: BtStage115Backend>(
    input:u32,
    ptr_pair:u32,
    state:u32,
    incoming_r3:u32,
    backend:&mut B,
)->u32 {
    let input14_first = backend.read_byte(input.wrapping_add(0x14));
    let mut regs = backend.boundary_335ac(
        u32::from(input14_first), ptr_pair, state, incoming_r3
    );
    let saved_first_r0 = regs.r0;

    let input14_second = backend.read_byte(input.wrapping_add(0x14));
    regs.r0 = u32::from(input14_second);
    regs = backend.boundary_1ee18(regs.r0, regs.r1, regs.r2, regs.r3);
    let object = regs.r0;

    let state1c = backend.read_byte(state.wrapping_add(0x1C));
    regs.r3 = u32::from(state1c);
    if state1c == 2 {
        let state1d = backend.read_byte(state.wrapping_add(0x1D));
        regs.r3 = u32::from(state1d);
        if state1d == 2 || state1d == 4 {
            let raw0b = backend.read_byte(object.wrapping_add(0x0B));
            regs.r3 = u32::from(raw0b);
            regs.r2 = selector4(raw0b);

            if regs.r2 <= 2 {
                regs.r2 = 3;
                regs.r3 = bfi_u32(regs.r3, regs.r2, 2, 4);
                backend.write_byte(object.wrapping_add(0x0B), regs.r3 as u8);
                return backend.tail_1f3e0(
                    object, saved_first_r0, regs.r2, regs.r3
                ).r0;
            }

            if regs.r2 == 3 {
                regs.r0 = object;
                regs = backend.boundary_1f3bc(regs.r0, regs.r1, regs.r2, regs.r3);
                if regs.r0 == 0 {
                    return 0;
                }
                let fresh0b = backend.read_byte(object.wrapping_add(0x0B));
                regs.r3 = u32::from(fresh0b | 0x3C);
                backend.write_byte(object.wrapping_add(0x0B), regs.r3 as u8);
                return regs.r0;
            }

            return object;
        }
    }

    let input14_third = backend.read_byte(input.wrapping_add(0x14));
    regs.r3 = u32::from(input14_third);
    backend.write_byte(object.wrapping_add(0x0C), input14_third);

    let first_a7 = backend.read_byte(saved_first_r0.wrapping_add(0xA7));
    regs.r3 = u32::from(first_a7 & 0xE0);
    let input0_for_mode = backend.read_byte(input);
    regs.r2 = u32::from(input0_for_mode);

    let mut raw0b = backend.read_byte(object.wrapping_add(0x0B));
    regs.r3 = u32::from(raw0b);

    let mode = if first_a7 & 0xE0 == 0x20 {
        let n = (u32::from(input0_for_mode) >> 3) & 0x0F;
        if n <= 9 { 1u32 } else { 2u32 }
    } else {
        let masked = u32::from(input0_for_mode) & 0x78;
        if masked == 0x18 || masked == 0x48 { 1u32 } else { 2u32 }
    };
    regs.r2 = mode;
    regs.r3 = bfi_u32(regs.r3, regs.r2, 6, 2);
    backend.write_byte(object.wrapping_add(0x0B), regs.r3 as u8);

    let dst1 = backend.read_word(ptr_pair.wrapping_add(4));
    regs.r2 = dst1;
    let global_first = backend.read_global_word_3189dc();
    regs.r1 = global_first;
    backend.write_byte(dst1, global_first as u8);

    let global_second = backend.read_global_word_3189dc();
    regs.r3 = global_second >> 8;
    let dst0 = backend.read_word(ptr_pair);
    regs.r2 = dst0;
    backend.write_byte(dst0, regs.r3 as u8);

    raw0b = backend.read_byte(object.wrapping_add(0x0B));
    regs.r3 = selector4(raw0b);
    let selector = regs.r3 as u8;

    match selector {
        0 | 1 => {
            let word10 = backend.read_word(object.wrapping_add(0x10));
            regs.r2 = word10;
            if word10 != 0 {
                regs.r0 = input;
                regs.r1 = ptr_pair;
                regs.r2 = state;
                return backend.tail_stage114(regs.r0, regs.r1, regs.r2, regs.r3).r0;
            }

            regs.r2 = u32::from(backend.read_byte(object.wrapping_add(0x0B)));
            regs.r3 = (regs.r3.wrapping_add(1)) & 0x0F;
            regs.r2 = bfi_u32(regs.r2, regs.r3, 2, 4);
            backend.write_byte(object.wrapping_add(0x0B), regs.r2 as u8);

            regs.r2 = 0x0C;
            let record = object.wrapping_add(regs.r2.wrapping_mul(regs.r3));
            regs.r3 = record;
            let occupied = backend.read_word(record.wrapping_add(0x14));
            regs.r2 = occupied;
            if occupied != 0 {
                regs.r0 = backend.read_word(ptr_pair);
                return backend.tail_b0460(regs.r0, regs.r1, regs.r2, regs.r3).r0;
            }

            backend.write_halfword(record.wrapping_add(0x18), 0);
            let input0 = backend.read_byte(input);
            regs.r2 = (u32::from(input0) >> 3) & 0x0F;
            let old18 = backend.read_byte(record.wrapping_add(0x18));
            regs.r0 = bfi_u32(u32::from(old18), regs.r2, 3, 4);
            backend.write_byte(record.wrapping_add(0x18), regs.r0 as u8);

            let input2 = backend.read_halfword(input.wrapping_add(2));
            regs.r2 = u32::from(input2);
            backend.write_halfword(record.wrapping_add(0x1A), input2);

            let value0 = backend.read_word(ptr_pair);
            regs.r2 = value0;
            backend.write_word(record.wrapping_add(0x14), value0);

            regs.r0 = object;
            regs.r1 = saved_first_r0;
            return backend.tail_1f3e0(regs.r0, regs.r1, regs.r2, regs.r3).r0;
        }

        2 => {
            let word10 = backend.read_word(object.wrapping_add(0x10));
            regs.r3 = word10;
            if word10 != 0 {
                regs.r0 = input;
                regs.r1 = ptr_pair;
                regs.r2 = state;
                return backend.tail_stage114(regs.r0, regs.r1, regs.r2, regs.r3).r0;
            }

            let word20 = backend.read_word(object.wrapping_add(0x20));
            regs.r2 = word20;
            let word2c = backend.read_word(object.wrapping_add(0x2C));
            regs.r3 = word2c;
            if word20 == 0 {
                if word2c != 0 {
                    regs.r0 = backend.read_word(ptr_pair);
                    return backend.tail_b0460(regs.r0, regs.r1, regs.r2, regs.r3).r0;
                }

                backend.write_halfword(object.wrapping_add(0x24), 0);
                let input0 = backend.read_byte(input);
                regs.r3 = (u32::from(input0) >> 3) & 0x0F;
                let old24 = backend.read_byte(object.wrapping_add(0x24));
                regs.r2 = bfi_u32(u32::from(old24), regs.r3, 3, 4);
                backend.write_byte(object.wrapping_add(0x24), regs.r2 as u8);

                let input2 = backend.read_halfword(input.wrapping_add(2));
                regs.r3 = u32::from(input2);
                backend.write_halfword(object.wrapping_add(0x26), input2);

                let value0 = backend.read_word(ptr_pair);
                regs.r3 = value0;
                backend.write_word(object.wrapping_add(0x20), value0);

                regs.r2 = 1;
                let fresh0b = backend.read_byte(object.wrapping_add(0x0B));
                regs.r3 = bfi_u32(u32::from(fresh0b), regs.r2, 2, 4);
                backend.write_byte(object.wrapping_add(0x0B), regs.r3 as u8);

                regs.r0 = object;
                regs.r1 = saved_first_r0;
                return backend.tail_1f3e0(regs.r0, regs.r1, regs.r2, regs.r3).r0;
            }

            if word2c != 0 {
                regs.r0 = backend.read_word(ptr_pair);
                return backend.tail_b0460(regs.r0, regs.r1, regs.r2, regs.r3).r0;
            }

            backend.write_halfword(object.wrapping_add(0x30), 0);
            let input0 = backend.read_byte(input);
            regs.r3 = (u32::from(input0) >> 3) & 0x0F;
            let old30 = backend.read_byte(object.wrapping_add(0x30));
            regs.r2 = bfi_u32(u32::from(old30), regs.r3, 3, 4);
            backend.write_byte(object.wrapping_add(0x30), regs.r2 as u8);

            let input2 = backend.read_halfword(input.wrapping_add(2));
            regs.r3 = u32::from(input2);
            backend.write_halfword(object.wrapping_add(0x32), input2);

            let value0 = backend.read_word(ptr_pair);
            regs.r3 = value0;
            backend.write_word(object.wrapping_add(0x2C), value0);

            regs.r0 = object;
            regs.r1 = saved_first_r0;
            return backend.tail_1f3e0(regs.r0, regs.r1, regs.r2, regs.r3).r0;
        }

        3 => {
            regs.r0 = object;
            regs = backend.boundary_1f3bc(regs.r0, regs.r1, regs.r2, regs.r3);
            if regs.r0 == 0 {
                regs.r0 = backend.read_word(ptr_pair);
                return backend.tail_b0460(regs.r0, regs.r1, regs.r2, regs.r3).r0;
            }

            regs.r2 = 0;
            backend.write_halfword(object.wrapping_add(0x18), 0);

            let input0 = backend.read_byte(input);
            regs.r3 = (u32::from(input0) >> 3) & 0x0F;
            let old18 = backend.read_byte(object.wrapping_add(0x18));
            regs.r1 = bfi_u32(u32::from(old18), regs.r3, 3, 4);
            backend.write_byte(object.wrapping_add(0x18), regs.r1 as u8);

            let input2 = backend.read_halfword(input.wrapping_add(2));
            regs.r3 = u32::from(input2);
            backend.write_halfword(object.wrapping_add(0x1A), input2);

            let value0 = backend.read_word(ptr_pair);
            regs.r3 = value0;
            backend.write_word(object.wrapping_add(0x14), value0);

            let fresh0b = backend.read_byte(object.wrapping_add(0x0B));
            regs.r3 = bfi_u32(u32::from(fresh0b), regs.r2, 2, 4);
            backend.write_byte(object.wrapping_add(0x0B), regs.r3 as u8);

            regs.r0 = object;
            regs.r1 = saved_first_r0;
            return backend.tail_1f3e0(regs.r0, regs.r1, regs.r2, regs.r3).r0;
        }

        15 => {
            let raw = backend.read_byte(object.wrapping_add(0x0B));
            regs.r3 = bfi_u32(u32::from(raw), 0, 2, 4);
            backend.write_byte(object.wrapping_add(0x0B), regs.r3 as u8);

            backend.write_halfword(object.wrapping_add(0x18), 0);

            let input0 = backend.read_byte(input);
            regs.r3 = (u32::from(input0) >> 3) & 0x0F;
            let old18 = backend.read_byte(object.wrapping_add(0x18));
            regs.r2 = bfi_u32(u32::from(old18), regs.r3, 3, 4);
            backend.write_byte(object.wrapping_add(0x18), regs.r2 as u8);

            let input2 = backend.read_halfword(input.wrapping_add(2));
            regs.r3 = u32::from(input2);
            backend.write_halfword(object.wrapping_add(0x1A), input2);

            let value0 = backend.read_word(ptr_pair);
            regs.r3 = value0;
            backend.write_word(object.wrapping_add(0x14), value0);

            regs.r0 = object;
            regs.r1 = saved_first_r0;
            return backend.tail_1f3e0(regs.r0, regs.r1, regs.r2, regs.r3).r0;
        }

        4..=14 => object,
        _ => object,
    }
}

#[cfg(test)]
mod stage115_tests {
    use super::*;
    use std::collections::{BTreeMap,VecDeque};
    use std::vec::Vec;

    #[derive(Clone,Debug,PartialEq,Eq)]
    enum E {
        R8(u32,u8),R16(u32,u16),R32(u32,u32),
        W8(u32,u8),W16(u32,u16),W32(u32,u32),
        G(u32),
        Call(&'static str,u32,u32,u32,u32),
    }

    struct B {
        b8:BTreeMap<u32,VecDeque<u8>>,
        h16:BTreeMap<u32,VecDeque<u16>>,
        w32:BTreeMap<u32,VecDeque<u32>>,
        globals:VecDeque<u32>,
        q335:VecDeque<BtStage115Regs>,
        qresolve:VecDeque<BtStage115Regs>,
        qgate:VecDeque<BtStage115Regs>,
        qstage114:VecDeque<BtStage115Regs>,
        qdestroy:VecDeque<BtStage115Regs>,
        qfinal:VecDeque<BtStage115Regs>,
        e:Vec<E>,
    }

    impl Default for B {
        fn default()->Self {
            let mut b=Self{
                b8:BTreeMap::new(),h16:BTreeMap::new(),w32:BTreeMap::new(),
                globals:VecDeque::from([0x11223344,0x11223344]),
                q335:VecDeque::from([BtStage115Regs{r0:0x5000,r1:0x11,r2:0x22,r3:0x33}]),
                qresolve:VecDeque::from([BtStage115Regs{r0:0x2000,r1:0x41,r2:0x42,r3:0x43}]),
                qgate:VecDeque::from([BtStage115Regs{r0:1,r1:0x51,r2:0x52,r3:0x53}]),
                qstage114:VecDeque::from([BtStage115Regs{r0:0x114,..Default::default()}]),
                qdestroy:VecDeque::from([BtStage115Regs{r0:0xD00D,..Default::default()}]),
                qfinal:VecDeque::from([BtStage115Regs{r0:0xF3E0,..Default::default()}]),
                e:Vec::new(),
            };
            b.push8(0x1014,[0x14,0x14,0x14]);
            b.push8(0x301C,[0]); b.push8(0x301D,[0]);
            b.push8(0x50A7,[0x20]);
            b.push8(0x1000,[0x18,0x18]);
            b.push16(0x1002,[0x5678]);
            b.push8(0x200B,[0,0,0]);
            b.push32(0x4000,[0x6000,0x6000,0x6000]);
            b.push32(0x4004,[0x6004]);
            b
        }
    }
    impl B{
        fn push8<const N:usize>(&mut self,a:u32,v:[u8;N]){self.b8.insert(a,VecDeque::from(v));}
        fn push16<const N:usize>(&mut self,a:u32,v:[u16;N]){self.h16.insert(a,VecDeque::from(v));}
        fn push32<const N:usize>(&mut self,a:u32,v:[u32;N]){self.w32.insert(a,VecDeque::from(v));}
        fn pop(q:&mut VecDeque<BtStage115Regs>)->BtStage115Regs{q.pop_front().unwrap()}
    }
    impl BtStage115Backend for B {
        fn read_byte(&mut self,a:u32)->u8{let v=self.b8.get_mut(&a).unwrap_or_else(||panic!("no b8 {a:#x}")).pop_front().unwrap();self.e.push(E::R8(a,v));v}
        fn read_halfword(&mut self,a:u32)->u16{let v=self.h16.get_mut(&a).unwrap_or_else(||panic!("no h16 {a:#x}")).pop_front().unwrap();self.e.push(E::R16(a,v));v}
        fn read_word(&mut self,a:u32)->u32{let v=self.w32.get_mut(&a).unwrap_or_else(||panic!("no w32 {a:#x}")).pop_front().unwrap();self.e.push(E::R32(a,v));v}
        fn write_byte(&mut self,a:u32,v:u8){self.e.push(E::W8(a,v))}
        fn write_halfword(&mut self,a:u32,v:u16){self.e.push(E::W16(a,v))}
        fn write_word(&mut self,a:u32,v:u32){self.e.push(E::W32(a,v))}
        fn read_global_word_3189dc(&mut self)->u32{let v=self.globals.pop_front().unwrap();self.e.push(E::G(v));v}
        fn boundary_335ac(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage115Regs{self.e.push(E::Call("335ac",a,b,c,d));Self::pop(&mut self.q335)}
        fn boundary_1ee18(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage115Regs{self.e.push(E::Call("1ee18",a,b,c,d));Self::pop(&mut self.qresolve)}
        fn boundary_1f3bc(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage115Regs{self.e.push(E::Call("1f3bc",a,b,c,d));Self::pop(&mut self.qgate)}
        fn tail_stage114(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage115Regs{self.e.push(E::Call("stage114",a,b,c,d));Self::pop(&mut self.qstage114)}
        fn tail_b0460(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage115Regs{self.e.push(E::Call("b0460",a,b,c,d));Self::pop(&mut self.qdestroy)}
        fn tail_1f3e0(&mut self,a:u32,b:u32,c:u32,d:u32)->BtStage115Regs{self.e.push(E::Call("1f3e0",a,b,c,d));Self::pop(&mut self.qfinal)}
    }

    fn seed_common(b:&mut B, selector:u8) {
        let raw=(selector & 0xF)<<2;
        b.b8.insert(0x200B,VecDeque::from([0,raw|0x40,raw|0x40]));
    }

    #[test]
    fn entry_boundaries_get_exact_args_and_special_selector_gt3_returns_object(){
        let mut b=B::default();
        b.b8.insert(0x301C,VecDeque::from([2]));b.b8.insert(0x301D,VecDeque::from([2]));
        b.b8.insert(0x200B,VecDeque::from([4<<2]));
        assert_eq!(bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0x99,&mut b),0x2000);
        assert!(b.e.contains(&E::Call("335ac",0x14,0x4000,0x3000,0x99)));
        assert!(b.e.contains(&E::Call("1ee18",0x14,0x11,0x22,0x33)));
    }

    #[test]
    fn special_selector_le2_sets_three_and_tails_final(){
        let mut b=B::default();
        b.b8.insert(0x301C,VecDeque::from([2]));b.b8.insert(0x301D,VecDeque::from([4]));
        b.b8.insert(0x200B,VecDeque::from([0x81]));
        assert_eq!(bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b),0xF3E0);
        let rewritten=(0x81u32 & !0x3C) | (3<<2);
        assert!(b.e.contains(&E::W8(0x200B,rewritten as u8)));
        assert!(b.e.contains(&E::Call("1f3e0",0x2000,0x5000,3,rewritten)));
    }

    #[test]
    fn special_selector3_gate_zero_returns_zero_and_nonzero_sets_0x3c(){
        let mut b=B::default();
        b.b8.insert(0x301C,VecDeque::from([2]));b.b8.insert(0x301D,VecDeque::from([2]));
        b.b8.insert(0x200B,VecDeque::from([3<<2]));
        b.qgate=VecDeque::from([BtStage115Regs{r0:0,r1:1,r2:2,r3:3}]);
        assert_eq!(bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b),0);

        let mut b=B::default();
        b.b8.insert(0x301C,VecDeque::from([2]));b.b8.insert(0x301D,VecDeque::from([2]));
        b.b8.insert(0x200B,VecDeque::from([3<<2,0x80]));
        b.qgate=VecDeque::from([BtStage115Regs{r0:7,r1:1,r2:2,r3:3}]);
        assert_eq!(bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b),7);
        assert!(b.e.contains(&E::W8(0x200B,0xBC)));
    }

    #[test]
    fn common_prefix_rewrites_mode_and_copies_two_true_global_reads(){
        let mut b=B::default(); seed_common(&mut b,4);
        b.globals=VecDeque::from([0x11223344,0x55667788]);
        let out=bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b);
        assert_eq!(out,0x2000);
        assert!(b.e.contains(&E::W8(0x200C,0x14)));
        assert!(b.e.contains(&E::W8(0x6004,0x44)));
        assert!(b.e.contains(&E::W8(0x6000,0x77)));
    }

    #[test]
    fn selector0_word10_nonzero_tails_stage114_with_selector_r3(){
        let mut b=B::default(); seed_common(&mut b,0);
        b.push32(0x2010,[9]);
        assert_eq!(bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b),0x114);
        assert!(b.e.contains(&E::Call("stage114",0x1000,0x4000,0x3000,0)));
    }

    #[test]
    fn selector0_empty_next_record_initializes_and_tails_with_record_pointer(){
        let mut b=B::default(); seed_common(&mut b,0);
        b.push32(0x2010,[0]);
        let rec=0x2000+0x0C;
        b.push32(rec+0x14,[0]);
        b.push8(rec+0x18,[0]);
        assert_eq!(bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b),0xF3E0);
        assert!(b.e.contains(&E::W16(rec+0x18,0)));
        assert!(b.e.contains(&E::W16(rec+0x1A,0x5678)));
        assert!(b.e.contains(&E::W32(rec+0x14,0x6000)));
        assert!(b.e.contains(&E::Call("1f3e0",0x2000,0x5000,0x6000,rec)));
    }

    #[test]
    fn selector0_occupied_record_tails_destroy_with_global_word_and_record_ptr(){
        let mut b=B::default(); seed_common(&mut b,0);
        b.push32(0x2010,[0]);
        let rec=0x2000+0x0C;
        b.push32(rec+0x14,[0xAABB]);
        assert_eq!(bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b),0xD00D);
        assert!(b.e.contains(&E::Call("b0460",0x6000,0x11223344,0xAABB,rec)));
    }

    #[test]
    fn selector2_word10_nonzero_tails_stage114_with_word10_in_r3(){
        let mut b=B::default(); seed_common(&mut b,2);
        b.push32(0x2010,[0xABCD]);
        let _=bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b);
        assert!(b.e.contains(&E::Call("stage114",0x1000,0x4000,0x3000,0xABCD)));
    }

    #[test]
    fn selector2_resource_pair_tail_and_first_record_initialization_are_exact(){
        let mut b=B::default(); seed_common(&mut b,2);
        b.push32(0x2010,[0]);b.push32(0x2020,[0x20]);b.push32(0x202C,[0x2C]);
        let _=bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b);
        assert!(b.e.contains(&E::Call("b0460",0x6000,0x11223344,0x20,0x2C)));

        let mut b=B::default(); seed_common(&mut b,2);
        b.push32(0x2010,[0]);b.push32(0x2020,[0]);b.push32(0x202C,[0]);
        b.push8(0x2024,[0]);
        let _=bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b);
        assert!(b.e.contains(&E::W16(0x2024,0)));
        assert!(b.e.contains(&E::W32(0x2020,0x6000)));
        assert!(b.e.contains(&E::Call("1f3e0",0x2000,0x5000,1,0x44)));

        let mut b=B::default(); seed_common(&mut b,2);
        b.push32(0x2010,[0]);b.push32(0x2020,[0]);b.push32(0x202C,[0x2C]);
        let _=bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b);
        assert!(b.e.contains(&E::Call("b0460",0x6000,0x11223344,0,0x2C)));
    }

    #[test]
    fn selector2_occupied_first_empty_second_initializes_second_record(){
        let mut b=B::default(); seed_common(&mut b,2);
        b.push32(0x2010,[0]);b.push32(0x2020,[0x20]);b.push32(0x202C,[0]);
        b.push8(0x2030,[0]);
        let _=bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b);
        assert!(b.e.contains(&E::W16(0x2030,0)));
        assert!(b.e.contains(&E::W16(0x2032,0x5678)));
        assert!(b.e.contains(&E::W32(0x202C,0x6000)));
        assert!(b.e.contains(&E::Call("1f3e0",0x2000,0x5000,0x18,0x6000)));
    }

    #[test]
    fn selector3_gate_zero_tail_preserves_boundary_return_tuple(){
        let mut b=B::default(); seed_common(&mut b,3);
        b.qgate=VecDeque::from([BtStage115Regs{r0:0,r1:0xA1,r2:0xA2,r3:0xA3}]);
        let _=bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b);
        assert!(b.e.contains(&E::Call("1f3bc",0x2000,0x11223344,0x6000,3)));
        assert!(b.e.contains(&E::Call("b0460",0x6000,0xA1,0xA2,0xA3)));
    }

    #[test]
    fn selector3_gate_nonzero_initializes_selector0_and_tails_final(){
        let mut b=B::default(); seed_common(&mut b,3);
        b.qgate=VecDeque::from([BtStage115Regs{r0:1,r1:0xA1,r2:0xA2,r3:0xA3}]);
        b.push8(0x2018,[0]);
        let _=bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b);
        assert!(b.e.contains(&E::W16(0x2018,0)));
        assert!(b.e.contains(&E::W32(0x2014,0x6000)));
        assert!(b.e.iter().any(|e|matches!(e,E::Call("1f3e0",0x2000,0x5000,0,_))));
    }

    #[test]
    fn selector15_clears_selector_and_initializes_primary_record(){
        let mut b=B::default(); seed_common(&mut b,15);
        b.push8(0x2018,[0]);
        let _=bt_stage115_selector_dispatch(0x1000,0x4000,0x3000,0,&mut b);
        assert!(b.e.contains(&E::W16(0x2018,0)));
        assert!(b.e.contains(&E::W32(0x2014,0x6000)));
        assert!(b.e.iter().any(|e|matches!(e,E::Call("1f3e0",0x2000,0x5000,_,0x6000))));
    }

    #[test]
    fn provenance_constants_are_exact(){
        assert_eq!(STAGE115_CURRENT_BT_SELECTOR_DISPATCH_ADDR,0x16DF10);
        assert_eq!(STAGE115_LEGACY_BT_SELECTOR_DISPATCH_ADDR,0x16AF44);
        assert_eq!(STAGE115_CURRENT_BODY_LEN,450);
        assert_eq!(STAGE115_GLOBAL_WORD_ADDR,0x3189DC);
        assert_eq!(STAGE115_BT_FIRST_BOUNDARY,0x335AC);
        assert_eq!(STAGE115_BT_RESOLVE_BOUNDARY,0x1EE18);
        assert_eq!(STAGE115_BT_GATE_BOUNDARY,0x1F3BC);
        assert_eq!(STAGE115_BT_DESTROY_TAIL,0xB0460);
        assert_eq!(STAGE115_BT_FINAL_TAIL,0x1F3E0);
    }
}

pub const STAGE116_CURRENT_ADDR:u32 = 0x0016_E0D8;
pub const STAGE116_LEGACY_ADDR:u32 = 0x0016_B10C;
pub const STAGE116_BODY_LEN:u32 = 1410;
pub const STAGE116_INLINE_LITERAL_START:u32 = 0x0016_E380;
pub const STAGE116_INLINE_LITERAL_END:u32 = 0x0016_E3A8;
pub const STAGE116_POST_BODY_NOP:u32 = 0x0016_E65A;
pub const STAGE116_POST_BODY_LITERAL_START:u32 = 0x0016_E65C;
pub const STAGE116_POST_BODY_LITERAL_END:u32 = 0x0016_E6B0;

pub const STAGE116_STACK_GUARD_WORD:u32 = 0x0020_0890;
pub const STAGE116_FLAGS_206F78:u32 = 0x0020_6F78;
pub const STAGE116_FLAGS_208338:u32 = 0x0020_8338;
pub const STAGE116_FLAGS_209B98:u32 = 0x0020_9B98;
pub const STAGE116_PRIMARY_208B78:u32 = 0x0020_8B78;
pub const STAGE116_MASK_209B94:u32 = 0x0020_9B94;
pub const STAGE116_THRESHOLD_2079B6:u32 = 0x0020_79B6;
pub const STAGE116_SNAPSHOT_318ACC:u32 = 0x0031_8ACC;
pub const STAGE116_MODE_TABLE_20289E:u32 = 0x0020_289E;
pub const STAGE116_MODE_THRESHOLD_202854:u32 = 0x0020_2854;
pub const STAGE116_OPTIONAL_215C20:u32 = 0x0021_5C20;
pub const STAGE116_FLAGS_202FA8:u32 = 0x0020_2FA8;
pub const STAGE116_MASK_208BB8:u32 = 0x0020_8BB8;
pub const STAGE116_MATRIX_INDEX_208C98:u32 = 0x0020_8C98;
pub const STAGE116_MATRIX_BASE_208C9C:u32 = 0x0020_8C9C;
pub const STAGE116_G_202868:u32 = 0x0020_2868;
pub const STAGE116_G_208B74:u32 = 0x0020_8B74;
pub const STAGE116_G_202852:u32 = 0x0020_2852;
pub const STAGE116_G_208B6D:u32 = 0x0020_8B6D;
pub const STAGE116_G_202865:u32 = 0x0020_2865;
pub const STAGE116_G_202866:u32 = 0x0020_2866;
pub const STAGE116_G_207BA8:u32 = 0x0020_7BA8;
pub const STAGE116_G_207BA5:u32 = 0x0020_7BA5;
pub const STAGE116_G_208BBC:u32 = 0x0020_8BBC;
pub const STAGE116_G_20285B:u32 = 0x0020_285B;
pub const STAGE116_G_206FE0:u32 = 0x0020_6FE0;
pub const STAGE116_G_3186D0:u32 = 0x0031_86D0;
pub const STAGE116_G_207FC1:u32 = 0x0020_7FC1;
pub const STAGE116_G_20B278:u32 = 0x0020_B278;

pub const STAGE116_CALL_3A604:u32=0x0003_A604;
pub const STAGE116_CALL_17E2C:u32=0x0001_7E2C;
pub const STAGE116_CALL_17820:u32=0x0001_7820;
pub const STAGE116_CALL_390E4:u32=0x0003_90E4;
pub const STAGE116_CALL_202E8:u32=0x0002_02E8;
pub const STAGE116_CALL_25288:u32=0x0002_5288;
pub const STAGE116_CALL_1D104:u32=0x0001_D104;
pub const STAGE116_CALL_2521C:u32=0x0002_521C;
pub const STAGE116_CALL_6301C:u32=0x0006_301C;
pub const STAGE116_CALL_4D57C:u32=0x0004_D57C;
pub const STAGE116_CALL_21FC2:u32=0x0002_1FC2;
pub const STAGE116_CALL_6304C:u32=0x0006_304C;
pub const STAGE116_CALL_29778:u32=0x0002_9778;
pub const STAGE116_CALL_4D4DC:u32=0x0004_D4DC;
pub const STAGE116_CALL_1EFA8:u32=0x0001_EFA8;
pub const STAGE116_CALL_367CC:u32=0x0003_67CC;
pub const STAGE116_CALL_AF094:u32=0x000A_F094;
pub const STAGE116_CALL_624E8:u32=0x0006_24E8;
pub const STAGE116_CALL_253B0:u32=0x0002_53B0;
pub const STAGE116_CALL_335AC:u32=0x0003_35AC;
pub const STAGE116_CALL_38918:u32=0x0003_8918;
pub const STAGE116_CALL_3C7C2:u32=0x0003_C7C2;
pub const STAGE116_CALL_2C79E:u32=0x0002_C79E;
pub const STAGE116_CALL_2C78C:u32=0x0002_C78C;
pub const STAGE116_CALL_629D0:u32=0x0006_29D0;
pub const STAGE116_CALL_1EBA0:u32=0x0001_EBA0;
pub const STAGE116_CALL_6329C:u32=0x0006_329C;
pub const STAGE116_CALL_2CAA8:u32=0x0002_CAA8;
pub const STAGE116_CALL_3A742:u32=0x0003_A742;
pub const STAGE116_CALL_25320:u32=0x0002_5320;
pub const STAGE116_CALL_94C0:u32=0x0000_94C0;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage116Regs { pub r0:u32,pub r1:u32,pub r2:u32,pub r3:u32 }

pub trait BtStage116Backend {
    fn read8(&mut self,addr:u32)->u8;
    fn read16(&mut self,addr:u32)->u16;
    fn read32(&mut self,addr:u32)->u32;
    fn write8(&mut self,addr:u32,value:u8);
    fn write16(&mut self,addr:u32,value:u16);
    fn write32(&mut self,addr:u32,value:u32);
    fn call(&mut self,target:u32,regs:BtStage116Regs)->BtStage116Regs;
    /// Current `0x3A742(4,state,&scratch_arg1,live_r3)`.
    /// R2 is an opaque real stack address in the binary, so it is represented by the
    /// mutable scratch reference rather than by a fabricated numeric register value.
    fn final_call(&mut self,r0:u32,r1:u32,scratch_arg1:&mut u32,r3:u32)->BtStage116Regs;
}

#[inline]
fn s116_bfi(dst:u32,src:u32,lsb:u32,width:u32)->u32 {
    let low=(1u32<<width)-1;
    let mask=low<<lsb;
    (dst & !mask) | ((src & low)<<lsb)
}
#[inline] fn s116_mode(v:u8)->u32 { (u32::from(v)>>3)&0x0f }
#[inline] fn s116_low16_replace(dst:u32,v:u16)->u32 { (dst&0xffff_0000)|u32::from(v) }
#[inline] fn s116_lo8(v:u32)->u8 { v as u8 }
#[inline] fn s116_hi8(v:u32)->u8 { (v>>8) as u8 }
#[inline] fn s116_set_lo8(v:u32,b:u8)->u32 { (v&!0xff)|u32::from(b) }
#[inline] fn s116_set_hi8(v:u32,b:u8)->u32 { (v&!0xff00)|(u32::from(b)<<8) }

fn s116_finish<B:BtStage116Backend>(
    backend:&mut B, saved_guard:u32, mut regs:BtStage116Regs
)->u32 {
    let now=backend.read32(STAGE116_STACK_GUARD_WORD);
    regs.r2=saved_guard;
    regs.r3=now;
    if regs.r2!=regs.r3 {
        regs=backend.call(STAGE116_CALL_94C0,regs);
    }
    regs.r0
}

fn s116_early_final<B:BtStage116Backend>(
    backend:&mut B, saved_guard:u32, mut regs:BtStage116Regs
)->u32 {
    regs=backend.call(STAGE116_CALL_202E8,regs);
    s116_finish(backend,saved_guard,regs)
}

fn s116_tail<B:BtStage116Backend>(
    state:u32,
    backend:&mut B,
    mut regs:BtStage116Regs,
    scratch_state:u32,
    scratch_arg1:&mut u32,
    saved_guard:u32,
)->u32 {
    regs.r3=u32::from(backend.read8(state.wrapping_add(0x0f)));
    if regs.r3==1 {
        regs.r3=STAGE116_G_208BBC;
        regs.r2=backend.read32(state.wrapping_add(0xf8));
        regs.r3=backend.read32(regs.r3);
        if regs.r2 & regs.r3 !=0 {
            regs.r3=u32::from(backend.read8(state.wrapping_add(0x99)));
            regs.r2=regs.r3<<31;
            if regs.r2 & 0x8000_0000 !=0 {
                regs.r3=STAGE116_G_20285B;
                regs.r3=u32::from(backend.read8(regs.r3));
                backend.write8(state.wrapping_add(0x131),regs.r3 as u8);
            }
        }
    }

    regs.r3=u32::from(backend.read8(STAGE116_FLAGS_208338.wrapping_add(0x13)));
    // Exact LSLS #28/BMI gate: source bit3 (0x08), not bit4.
    if regs.r3 & 0x08 !=0 {
        regs.r0=state;
        regs=backend.call(STAGE116_CALL_629D0,regs);
    }

    regs.r3=u32::from(backend.read8(STAGE116_G_206FE0.wrapping_add(9)));
    if regs.r3!=0 {
        regs.r1=scratch_state;
        regs.r0=u32::from(backend.read8(state.wrapping_add(0xa5)));
        regs=backend.call(STAGE116_CALL_1EBA0,regs);
    }

    regs.r0=state;
    regs=backend.call(STAGE116_CALL_6329C,regs);

    regs.r3=u32::from(backend.read8(state.wrapping_add(0x5e)));
    if regs.r3!=0 {
        regs.r3=u32::from(backend.read16(state.wrapping_add(0x9a)));
        regs.r3 &= !4;
        regs.r3 &= 0x1fff;
        let eq=regs.r3==1;
        regs.r3=STAGE116_G_3186D0;
        regs.r2=backend.read32(regs.r3);
        regs.r2=if eq {regs.r2|1}else{regs.r2&!1};
        backend.write32(regs.r3,regs.r2);
    }

    regs.r3=u32::from(backend.read8(STAGE116_G_207FC1));
    if regs.r3!=0 {
        regs.r3=u32::from(backend.read8(STAGE116_G_20B278));
        if regs.r3!=0 {
            regs.r1=1;
            regs.r0=state;
            regs=backend.call(STAGE116_CALL_2CAA8,regs);
        }
    }

    regs=backend.final_call(4,state,scratch_arg1,regs.r3);
    s116_finish(backend,saved_guard,regs)
}

/// Exact current-HCD register/memory model for `0x16E0D8..0x16E65A`.
///
/// Unlike the older Stage-49 semantic abstraction, this surface keeps complete
/// caller-volatile R0-R3 tuples across every opaque call, performs every ambient/global
/// reread at its actual program point, keeps the two aliased pushed stack dwords, and
/// preserves the exact current bit gates. The compiler stack-canary load/check is also
/// represented because its first value can be live in R3 at the first runtime boundary.
pub fn bt_stage116_register_state<B:BtStage116Backend>(
    state:u32,
    arg1:u32,
    incoming_r2:u32,
    incoming_r3:u32,
    backend:&mut B,
)->u32 {
    let mut regs=BtStage116Regs{r0:state,r1:arg1,r2:incoming_r2,r3:incoming_r3};
    let mut scratch_state=state;
    let mut scratch_arg1=arg1;
    let saved_arg1=arg1;

    regs.r3=backend.read32(STAGE116_STACK_GUARD_WORD);
    let saved_guard=regs.r3;

    if arg1==1 {
        regs.r3=0;
        backend.write8(state.wrapping_add(0x97),0);
        backend.write8(state.wrapping_add(0x115),0);
        backend.write8(state.wrapping_add(0x116),0);
    }

    regs=backend.call(STAGE116_CALL_3A604,regs);
    if regs.r0!=0 {
        regs.r0=u32::from(backend.read8(state.wrapping_add(0x0e)));
        regs=backend.call(STAGE116_CALL_17E2C,regs);
        regs.r1=1;
        regs=backend.call(STAGE116_CALL_17820,regs);
        regs.r2=u32::from(backend.read8(state.wrapping_add(0x0f)));
        regs.r1=u32::from(backend.read8(state.wrapping_add(0xa4)));
        regs.r2=u32::from(regs.r2==0);
        regs=backend.call(STAGE116_CALL_390E4,regs);
        if regs.r0==0 {
            return s116_early_final(backend,saved_guard,regs);
        }
    }

    regs=backend.call(STAGE116_CALL_25288,regs);
    if regs.r0==0 {
        regs.r3=u32::from(backend.read8(STAGE116_FLAGS_206F78.wrapping_add(0x16)));
        if regs.r3!=0 {
            regs.r1=0x18;
            regs.r0=0x32;
            regs=backend.call(STAGE116_CALL_1D104,regs);
        }
        return s116_early_final(backend,saved_guard,regs);
    }

    regs.r2=u32::from(backend.read8(state.wrapping_add(0x0f)));
    regs.r3=0;
    backend.write16(state.wrapping_add(0x0c),0);

    if regs.r2==saved_arg1 {
        backend.write8(state.wrapping_add(0x94),regs.r3 as u8);
        backend.write8(state.wrapping_add(0x95),regs.r3 as u8);
        regs=backend.call(STAGE116_CALL_25320,regs);
        regs.r3=u32::from(backend.read8(state.wrapping_add(0x5e)));
        if regs.r3!=0 {
            regs.r2=STAGE116_G_3186D0;
            regs.r3=backend.read32(regs.r2);
            regs.r3 &= !1;
            backend.write32(regs.r2,regs.r3);
        }
        regs.r3=backend.read32(state.wrapping_add(0x68));
        if regs.r3!=0 {
            regs.r3=u32::from(backend.read16(state.wrapping_add(0x78)));
            backend.write16(state.wrapping_add(0x0c),regs.r3 as u16);
        }
        return s116_finish(backend,saved_guard,regs);
    }

    regs=backend.call(STAGE116_CALL_2521C,regs);

    regs.r3=u32::from(backend.read8(state.wrapping_add(0x98)));
    if regs.r0!=0 {
        regs.r3|=0x80;
    } else {
        regs.r2=u32::from(backend.read8(STAGE116_FLAGS_208338.wrapping_add(0x13)))&8;
        regs.r1=regs.r2&0xff;
        if regs.r2!=0 { regs.r3|=0x80; }
        else { regs.r3=s116_bfi(regs.r3,regs.r1,7,1); }
    }
    backend.write8(state.wrapping_add(0x98),regs.r3 as u8);

    regs.r3=u32::from(backend.read8(STAGE116_FLAGS_209B98));
    regs.r2=regs.r3>>1;
    if regs.r2!=0 {
        regs.r2=regs.r3&1;
        regs.r3=u32::from(backend.read8(state.wrapping_add(0x98)));
        if regs.r2!=0 { regs.r3|=0x80; }
        else { regs.r3=s116_bfi(regs.r3,regs.r2,7,1); }
        backend.write8(state.wrapping_add(0x98),regs.r3 as u8);
    }

    regs.r0=state;
    regs=backend.call(STAGE116_CALL_6301C,regs);

    regs.r3=u32::from(backend.read8(STAGE116_FLAGS_208338.wrapping_add(0x13)));
    // Exact LSLS #28/BMI: bit3 set skips the state+0x90 boundary.
    if regs.r3 & 0x08 ==0 {
        regs.r0=state.wrapping_add(0x90);
        regs=backend.call(STAGE116_CALL_4D57C,regs);
    }

    let mut common=false;
    regs.r3=backend.read32(state.wrapping_add(0xa0));
    if regs.r3==0 {
        common=true;
    } else {
        regs.r3=u32::from(backend.read8(state.wrapping_add(0x90)));
        if regs.r3 & 0x80 ==0 {
            common=true;
        } else {
            regs.r3=u32::from(backend.read8(state.wrapping_add(0x135)));
            if regs.r3!=0 {
                common=true;
            } else {
                regs.r0=state;
                regs=backend.call(STAGE116_CALL_21FC2,regs);
                if regs.r0==0 {
                    regs.r3=u32::from(backend.read8(state.wrapping_add(0x0f)));
                    if regs.r3==1 {
                        regs.r3=STAGE116_PRIMARY_208B78;
                        regs.r2=u32::from(backend.read8(regs.r3.wrapping_add(0x3b)));
                        if regs.r2==0 {
                            regs.r3=backend.read32(regs.r3);
                            if state!=regs.r3 {
                                regs.r3=u32::from(backend.read8(state.wrapping_add(0x9e)));
                                if regs.r3==0 {
                                    common=true;
                                }
                            }
                        }
                    }
                }
                if !common {
                    regs=backend.call(STAGE116_CALL_6304C,regs);
                    if regs.r0!=0 { common=true; }
                }
            }
        }
    }

    if common {
        let role=backend.read8(state.wrapping_add(0x0f));
        regs.r3=u32::from(backend.read16(state.wrapping_add(0x98)));
        // This low16 overwrite exists only on the common path.
        scratch_arg1=s116_low16_replace(scratch_arg1,regs.r3 as u16);

        if role==1 {
            regs.r3=3;
            backend.write8(state.wrapping_add(0x11b),3);

            let b0=s116_lo8(scratch_arg1);
            regs.r3=u32::from(b0 & !0x78);
            scratch_arg1=s116_set_lo8(scratch_arg1,regs.r3 as u8);

            let b1=s116_hi8(scratch_arg1);
            regs.r0=state;
            regs.r3=u32::from(b1 & !0x7c);
            scratch_arg1=s116_set_hi8(scratch_arg1,regs.r3 as u8);

            regs=backend.call(STAGE116_CALL_21FC2,regs);
            if regs.r0==0 {
                regs.r2=STAGE116_PRIMARY_208B78;
                regs.r3=backend.read32(regs.r2);
                if state==regs.r3 {
                    regs.r3=STAGE116_MASK_209B94;
                    regs.r1=backend.read32(state.wrapping_add(0xf8));
                    regs.r3=backend.read32(regs.r3);
                    if regs.r1 & regs.r3 ==0 {
                        regs.r3=u32::from(backend.read8(state.wrapping_add(0x94)));
                        if regs.r3==2 {
                            regs.r3=u32::from(backend.read8(state.wrapping_add(0x90)));
                            regs.r3=(regs.r3>>3)&0x0f;
                            if regs.r3<=1 {
                                backend.write8(state.wrapping_add(0x124),role);
                                backend.write32(STAGE116_PRIMARY_208B78.wrapping_add(4),state);
                            }
                        }
                    }
                }
            }
        } else {
            let mut cleared=false;
            regs.r3=u32::from(backend.read8(state.wrapping_add(0x114)));
            if regs.r3!=0 {
                regs.r3=u32::from(backend.read16(state.wrapping_add(0x104)));
                if regs.r3>3 {
                    regs.r3=u32::from(backend.read8(state.wrapping_add(0x11b)));
                    if regs.r3>1 {
                        regs.r3=u32::from(backend.read8(state.wrapping_add(0x11e)));
                        if regs.r3==0 {
                            regs.r0=u32::from(backend.read8(state.wrapping_add(0x0e)));
                            regs=backend.call(STAGE116_CALL_29778,regs);
                            if regs.r0==0 {
                                cleared=true;
                            } else {
                                regs.r3=STAGE116_THRESHOLD_2079B6;
                                regs.r2=u32::from(backend.read16(state.wrapping_add(0x104)));
                                regs.r3=u32::from(backend.read16(regs.r3));
                                if regs.r2<regs.r3 { cleared=true; }
                            }
                        }
                    }
                }
            }

            if cleared {
                regs.r3=3;
                backend.write8(state.wrapping_add(0x11b),3);
                regs.r3=u32::from(s116_lo8(scratch_arg1)&!0x78);
                scratch_arg1=s116_set_lo8(scratch_arg1,regs.r3 as u8);
                regs.r3=u32::from(s116_hi8(scratch_arg1)&!0x7c);
                scratch_arg1=s116_set_hi8(scratch_arg1,regs.r3 as u8);
            } else {
                regs.r2=u32::from(s116_lo8(scratch_arg1));
                regs.r3=1;
                regs.r2=s116_bfi(regs.r2,regs.r3,3,4);
                scratch_arg1=s116_set_lo8(scratch_arg1,regs.r2 as u8);
                regs.r2=u32::from(s116_hi8(scratch_arg1));
                backend.write8(state.wrapping_add(0x11b),regs.r3 as u8);
                regs.r2=s116_bfi(regs.r2,regs.r3,2,5);
                scratch_arg1=s116_set_hi8(scratch_arg1,regs.r2 as u8);
            }
        }

        regs.r3=u32::from(scratch_arg1 as u16);
        scratch_state=s116_low16_replace(scratch_state,regs.r3 as u16);
        regs.r2=regs.r3;
        regs.r3=STAGE116_SNAPSHOT_318ACC;
        regs.r0=1;
        backend.write32(regs.r3,regs.r2);
        backend.write8(state.wrapping_add(0x9c),regs.r0 as u8);

        regs.r2=u32::from(backend.read8(STAGE116_FLAGS_208338.wrapping_add(0x13)))&8;
        regs.r3=regs.r2&0xff;
        if regs.r2==0 {
            regs.r1=u32::from(s116_lo8(scratch_arg1));
            regs.r2=regs.r3;
            regs.r1=(regs.r1>>3)&0x0f;
            regs=backend.call(STAGE116_CALL_4D4DC,regs);
        }

        regs.r3=0;
        backend.write8(state.wrapping_add(0x129),0);
        regs.r3=STAGE116_FLAGS_206F78;
        regs.r3=u32::from(backend.read8(regs.r3.wrapping_add(4)));
        if regs.r3!=0 {
            regs.r1=u32::from(scratch_arg1 as u16);
            regs.r0=1;
            regs.r1>>=3;
            regs=backend.call(STAGE116_CALL_1D104,regs);
        }

        return s116_tail(state,backend,regs,scratch_state,&mut scratch_arg1,saved_guard);
    }

    // Alternate path at current 0x16E304. scratch_arg1 is still the original arg1.
    regs.r3=u32::from(backend.read8(state.wrapping_add(0x0f)));
    if regs.r3==0 {
        regs.r3=1;
        backend.write8(state.wrapping_add(0x11b),1);
    }

    regs.r3=u32::from(backend.read8(state.wrapping_add(0x9f)));
    if regs.r3!=0 {
        regs.r3=0;
        backend.write8(state.wrapping_add(0x9f),0);
        regs.r3=u32::from(backend.read8(state.wrapping_add(0x99)));
        regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
        regs.r2=(regs.r3>>1)&1;
        regs.r2^=1;
        regs.r3=s116_bfi(regs.r3,regs.r2,1,1);
        backend.write8(state.wrapping_add(0x99),regs.r3 as u8);
        regs=backend.call(STAGE116_CALL_1EFA8,regs);
    }

    regs.r3=u32::from(backend.read8(state.wrapping_add(0x9e)));
    if regs.r3!=0 {
        regs.r3=u32::from(backend.read8(state.wrapping_add(0x134))).wrapping_add(1);
        backend.write8(state.wrapping_add(0x134),regs.r3 as u8);
        regs.r3=2;
        backend.write8(state.wrapping_add(0x129),2);

        regs.r3=s116_mode(backend.read8(state.wrapping_add(0x98)));
        regs.r2=u32::from(backend.read8(STAGE116_MODE_TABLE_20289E.wrapping_add(regs.r3)));
        regs.r3=backend.read32(STAGE116_MODE_THRESHOLD_202854);
        if regs.r2>regs.r3 {
            regs.r0=state;
            regs=backend.call(STAGE116_CALL_21FC2,regs);
            if regs.r0==0 {
                regs.r3=u32::from(backend.read8(state.wrapping_add(0xa4)));
                regs.r3=regs.r3.wrapping_sub(0x18);
                if regs.r3>2 {
                    regs.r0=state;
                    regs=backend.call(STAGE116_CALL_367CC,regs);
                    if regs.r0!=0 {
                        return s116_finish(backend,saved_guard,regs);
                    }
                }
            }
        }
    } else {
        regs.r3=0;
        backend.write8(state.wrapping_add(0x134),0);
        regs.r3=1;
        backend.write8(state.wrapping_add(0x129),1);
        regs.r3=backend.read32(STAGE116_OPTIONAL_215C20);
        if regs.r3!=0 {
            regs.r0=state;
            regs=backend.call(STAGE116_CALL_AF094,regs);
        }
    }

    regs.r3=1;
    backend.write8(state.wrapping_add(0x9e),1);
    backend.write8(state.wrapping_add(0x115),1);
    backend.write8(state.wrapping_add(0x125),1);

    regs.r3=backend.read32(STAGE116_FLAGS_202FA8);
    regs.r0=regs.r3<<15;
    if regs.r3 & (1<<16) !=0 {
        regs.r2=STAGE116_MASK_208BB8;
        regs.r1=backend.read32(state.wrapping_add(0xf8));
        regs.r3=backend.read32(regs.r2);
        regs.r3 &= !regs.r1;
        backend.write32(regs.r2,regs.r3);
    }

    regs.r3=u32::from(backend.read8(STAGE116_FLAGS_208338.wrapping_add(0x13)));
    regs.r1=regs.r3<<27;
    if regs.r3 & 0x10 !=0 {
        regs.r0=state;
        regs=backend.call(STAGE116_CALL_624E8,regs);
    }

    regs.r3=u32::from(backend.read16(state.wrapping_add(0x98)));
    scratch_state=s116_low16_replace(scratch_state,regs.r3 as u16);
    regs.r2=regs.r3;
    regs.r3=STAGE116_SNAPSHOT_318ACC;
    backend.write32(regs.r3,regs.r2);
    regs.r2=u32::from(backend.read16(state.wrapping_add(0x9a)));
    backend.write32(regs.r3.wrapping_add(4),regs.r2);

    regs.r0=state.wrapping_add(0x90);
    regs=backend.call(STAGE116_CALL_253B0,regs);

    regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
    regs=backend.call(STAGE116_CALL_335AC,regs);
    let context=regs.r0;
    if context!=0 {
        regs.r0=u32::from(backend.read16(context.wrapping_add(0x64)));
        regs=backend.call(STAGE116_CALL_38918,regs);
        if regs.r0!=2 {
            regs.r3=u32::from(backend.read8(context.wrapping_add(0xa7)))>>5;
            if regs.r3==0 {
                regs.r3=s116_mode(backend.read8(state.wrapping_add(0x98)));
                regs.r2=regs.r3&0x0b;
                let value=if regs.r2==0x0a || regs.r3==4 {Some(1u8)}
                    else if regs.r2==0x0b || regs.r3==8 {Some(0u8)} else {None};
                if let Some(v)=value {
                    regs.r3=backend.read32(STAGE116_MATRIX_INDEX_208C98);
                    regs.r2=0x28;
                    regs.r0=regs.r2.wrapping_mul(regs.r0);
                    regs.r2=0x14;
                    regs.r0=regs.r2.wrapping_mul(regs.r3).wrapping_add(regs.r0);
                    regs.r3=STAGE116_MATRIX_BASE_208C9C;
                    regs.r0=regs.r0.wrapping_add(regs.r3);
                    regs.r3=u32::from(backend.read8(regs.r0.wrapping_add(0x12)));
                    backend.write8(regs.r0.wrapping_add(0x13),regs.r3 as u8);
                    regs.r3=u32::from(v);
                    backend.write8(regs.r0.wrapping_add(0x12),v);
                }
            }
        }
    }

    regs.r3=s116_mode(backend.read8(state.wrapping_add(0x98)));
    regs.r3=u32::from(backend.read8(STAGE116_MODE_TABLE_20289E.wrapping_add(regs.r3)));
    backend.write8(state.wrapping_add(0x9c),regs.r3 as u8);
    regs.r3=regs.r3.wrapping_sub(1);
    backend.write16(state.wrapping_add(0x0c),regs.r3 as u16);

    regs.r3=u32::from(backend.read8(state.wrapping_add(0x0f)));
    if regs.r3==0 {
        regs.r0=state;
        regs=backend.call(STAGE116_CALL_21FC2,regs);
        if regs.r0==0 {
            regs.r3=u32::from(backend.read8(state.wrapping_add(0x121)));
            if regs.r3!=0 {
                regs.r0=state;
                regs=backend.call(STAGE116_CALL_3C7C2,regs);
                if regs.r0!=0 {
                    regs.r3=u32::from(backend.read8(STAGE116_G_202868));
                    backend.write8(state.wrapping_add(0x11e),regs.r3 as u8);
                    regs.r3=STAGE116_G_208B74;
                } else {
                    regs.r3=u32::from(backend.read8(STAGE116_G_202852));
                    backend.write8(state.wrapping_add(0x11e),regs.r3 as u8);
                    regs.r3=STAGE116_G_208B6D;
                }
            } else {
                regs.r3=u32::from(backend.read8(STAGE116_G_202865));
                backend.write8(state.wrapping_add(0x11e),regs.r3 as u8);
                regs.r3=STAGE116_G_202866;
            }
            regs.r3=u32::from(backend.read8(regs.r3));
            backend.write8(state.wrapping_add(0x11f),regs.r3 as u8);
        }
    }

    regs.r3=u32::from(backend.read8(STAGE116_FLAGS_206F78.wrapping_add(4)));
    if regs.r3!=0 {
        regs.r3=u32::from(backend.read16(state.wrapping_add(0x98)));
        regs.r1=u32::from(backend.read16(state.wrapping_add(0x9a)));
        regs.r3=(regs.r3>>3)&0x7f;
        regs.r1=regs.r3 | (regs.r1<<7);
        regs.r0=3;
        regs=backend.call(STAGE116_CALL_1D104,regs);

        regs.r3=u32::from(backend.read8(state.wrapping_add(0x129)));
        if regs.r3!=2 {
            regs.r1=1;
            regs.r0=0x65;
            regs=backend.call(STAGE116_CALL_1D104,regs);
        }
    }

    regs.r3=u32::from(backend.read8(STAGE116_FLAGS_206F78.wrapping_add(8)));
    if regs.r3!=0 {
        regs.r3=u32::from(backend.read8(state.wrapping_add(0x9a)))&3;
        if regs.r3==3 {
            regs.r3=u32::from(backend.read8(state.wrapping_add(0x129)));
            if regs.r3!=2 {
                regs.r3=backend.read32(state.wrapping_add(0xa0));
                regs.r1=u32::from(backend.read8(regs.r3));
                regs.r2=regs.r1&0xfe;
                if regs.r2==0xfe {
                    regs.r1=u32::from(backend.read8(regs.r3.wrapping_add(1))).wrapping_add(0x43);
                } else {
                    regs.r1>>=1;
                }
                regs.r0=0x5d;
                regs=backend.call(STAGE116_CALL_1D104,regs);
            }
        }
    }

    regs.r3=u32::from(backend.read8(STAGE116_FLAGS_206F78.wrapping_add(10)));
    if regs.r3!=0 {
        regs.r3=u32::from(backend.read8(state.wrapping_add(0x9a)))&3;
        if regs.r3!=3 {
            regs.r3=u32::from(backend.read8(state.wrapping_add(0xa4)))&0x0f;
            regs.r1=u32::from(backend.read16(state.wrapping_add(0x9a)));
            regs.r1=(regs.r1>>3)&0x03ff;
            regs.r1=regs.r3 | (regs.r1<<4);
            regs.r0=0x0d;
            regs=backend.call(STAGE116_CALL_1D104,regs);
        }
    }

    regs.r3=u32::from(backend.read8(STAGE116_G_207BA8));
    if regs.r3!=0 {
        regs.r1=s116_mode(backend.read8(state.wrapping_add(0x98)));
        regs.r0=u32::from(backend.read8(state.wrapping_add(0xa5)));
        regs=backend.call(STAGE116_CALL_2C79E,regs);
        if regs.r0!=0 {
            regs.r3=backend.read32(state.wrapping_add(0xa0));
            if regs.r3!=0 {
                regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
                regs=backend.call(STAGE116_CALL_335AC,regs);
                if regs.r0!=0 {
                    regs=backend.call(STAGE116_CALL_2C78C,regs);
                }
            }
        }

        regs.r3=s116_mode(backend.read8(state.wrapping_add(0x98)));
        if regs.r3>3 {
            regs.r3=backend.read32(state.wrapping_add(0xa0));
            if regs.r3!=0 {
                regs.r3=STAGE116_G_207BA5;
                regs.r2=1;
                backend.write8(regs.r3,1);
            }
        }
    }

    s116_tail(state,backend,regs,scratch_state,&mut scratch_arg1,saved_guard)
}

#[cfg(test)]
mod stage116_tests {
    use super::*;
    use std::collections::{BTreeMap,VecDeque};
    use std::vec::Vec;

    #[derive(Default)]
    struct B {
        mem:BTreeMap<u32,u8>,
        returns:BTreeMap<u32,VecDeque<BtStage116Regs>>,
        calls:Vec<(u32,BtStage116Regs)>,
        final_inputs:Vec<(u32,u32,u32,u32)> ,
    }
    impl B {
        fn set8(&mut self,a:u32,v:u8){self.mem.insert(a,v);}
        fn set16(&mut self,a:u32,v:u16){for(i,b)in v.to_le_bytes().iter().enumerate(){self.set8(a+i as u32,*b)}}
        fn set32(&mut self,a:u32,v:u32){for(i,b)in v.to_le_bytes().iter().enumerate(){self.set8(a+i as u32,*b)}}
        fn q(&mut self,t:u32,v:BtStage116Regs){self.returns.entry(t).or_default().push_back(v);}
        fn seed_guard(&mut self,v:u32){self.set32(STAGE116_STACK_GUARD_WORD,v);}
        fn call_seen(&self,t:u32)->bool{self.calls.iter().any(|x|x.0==t)}
    }
    impl BtStage116Backend for B {
        fn read8(&mut self,a:u32)->u8{*self.mem.get(&a).unwrap_or(&0)}
        fn read16(&mut self,a:u32)->u16{u16::from_le_bytes([self.read8(a),self.read8(a+1)])}
        fn read32(&mut self,a:u32)->u32{u32::from_le_bytes([self.read8(a),self.read8(a+1),self.read8(a+2),self.read8(a+3)])}
        fn write8(&mut self,a:u32,v:u8){self.mem.insert(a,v);}
        fn write16(&mut self,a:u32,v:u16){self.set16(a,v)}
        fn write32(&mut self,a:u32,v:u32){self.set32(a,v)}
        fn call(&mut self,t:u32,r:BtStage116Regs)->BtStage116Regs{
            self.calls.push((t,r));
            self.returns.get_mut(&t).and_then(|q|q.pop_front()).unwrap_or(r)
        }
        fn final_call(&mut self,r0:u32,r1:u32,s:&mut u32,r3:u32)->BtStage116Regs{
            self.final_inputs.push((r0,r1,*s,r3));
            self.returns.get_mut(&STAGE116_CALL_3A742).and_then(|q|q.pop_front())
                .unwrap_or(BtStage116Regs{r0,r1,r2:0,r3})
        }
    }

    fn seed_base(b:&mut B,state:u32,arg1:u32){
        b.seed_guard(0xA5A5_5A5A);
        b.set8(state+0x0f,(arg1 as u8).wrapping_add(1)); // avoid equal-arg reset
        b.set8(state+0x98,0);
        b.set8(state+0x99,0);
        b.set8(state+0x9a,0);
        b.set8(state+0x9b,0);
        b.set8(state+0xa4,9);
        b.set8(STAGE116_FLAGS_208338+0x13,0);
        b.set8(STAGE116_FLAGS_209B98,0);
        b.set32(STAGE116_PRIMARY_208B78,0);
        b.set32(STAGE116_MODE_THRESHOLD_202854,u32::MAX);
        b.set8(STAGE116_MODE_TABLE_20289E,1);
    }

    #[test]
    fn entry_one_zeros_three_bytes_and_passes_exact_initial_tuple(){
        let state=0x200000; let mut b=B::default(); seed_base(&mut b,state,1);
        b.set8(state+0x97,9);b.set8(state+0x115,8);b.set8(state+0x116,7);
        b.q(STAGE116_CALL_3A604,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_25288,BtStage116Regs{r0:0x10,r1:0x11,r2:0x12,r3:0x13});
        b.q(STAGE116_CALL_2521C,BtStage116Regs{r0:1,r1:0x21,r2:0x22,r3:0x23});
        let _=bt_stage116_register_state(state,1,0xCAFE,0xDEAD,&mut b);
        let first=b.calls.iter().find(|x|x.0==STAGE116_CALL_3A604).unwrap().1;
        assert_eq!(first,BtStage116Regs{r0:state,r1:1,r2:0xCAFE,r3:0});
        assert_eq!((b.read8(state+0x97),b.read8(state+0x115),b.read8(state+0x116)),(0,0,0));
    }

    #[test]
    fn non_one_entry_exposes_stack_guard_as_live_r3(){
        let state=0x210000;let mut b=B::default();seed_base(&mut b,state,3);
        b.q(STAGE116_CALL_3A604,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_25288,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_202E8,BtStage116Regs{r0:0x44,..Default::default()});
        assert_eq!(bt_stage116_register_state(state,3,0x1234,0x9999,&mut b),0x44);
        let first=b.calls.iter().find(|x|x.0==STAGE116_CALL_3A604).unwrap().1;
        assert_eq!(first.r2,0x1234);
        assert_eq!(first.r3,0xA5A5_5A5A);
    }

    #[test]
    fn bit3_not_bit4_controls_state_plus_90_pre_boundary(){
        let state=0x220000;let mut b=B::default();seed_base(&mut b,state,3);
        b.q(STAGE116_CALL_3A604,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_25288,BtStage116Regs{r0:1,..Default::default()});
        b.q(STAGE116_CALL_2521C,BtStage116Regs{r0:1,..Default::default()});
        b.set8(STAGE116_FLAGS_208338+0x13,0x08);
        b.set32(state+0xa0,0);
        let _=bt_stage116_register_state(state,3,0,0,&mut b);
        assert!(!b.call_seen(STAGE116_CALL_4D57C));

        let mut b=B::default();seed_base(&mut b,state,3);
        b.q(STAGE116_CALL_3A604,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_25288,BtStage116Regs{r0:1,..Default::default()});
        b.q(STAGE116_CALL_2521C,BtStage116Regs{r0:1,..Default::default()});
        b.set8(STAGE116_FLAGS_208338+0x13,0x10);
        b.set32(state+0xa0,0);
        let _=bt_stage116_register_state(state,3,0,0,&mut b);
        assert!(b.call_seen(STAGE116_CALL_4D57C));
    }

    #[test]
    fn alternate_path_keeps_original_scratch_arg1_and_skips_af094_when_byte9e_nonzero(){
        let state=0x230000;let arg1=0xABCD_1234;let mut b=B::default();seed_base(&mut b,state,arg1);
        b.set8(state+0x0f,2);
        b.set32(state+0xa0,1);
        b.set8(state+0x90,0x80);
        b.set8(state+0x135,0);
        b.set8(state+0x9e,1);
        b.set8(state+0x98,0);
        b.set32(STAGE116_OPTIONAL_215C20,1);
        b.q(STAGE116_CALL_3A604,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_25288,BtStage116Regs{r0:1,..Default::default()});
        b.q(STAGE116_CALL_2521C,BtStage116Regs{r0:1,..Default::default()});
        b.q(STAGE116_CALL_21FC2,BtStage116Regs{r0:1,..Default::default()}); // outer gate -> secondary
        b.q(STAGE116_CALL_6304C,BtStage116Regs{r0:0,..Default::default()}); // alternate
        b.q(STAGE116_CALL_3A742,BtStage116Regs{r0:0x55,..Default::default()});
        assert_eq!(bt_stage116_register_state(state,arg1,0,0,&mut b),0x55);
        assert!(!b.call_seen(STAGE116_CALL_AF094));
        let f=*b.final_inputs.last().unwrap();
        assert_eq!((f.0,f.1,f.2),(4,state,arg1));
    }

    #[test]
    fn byte9e_zero_is_the_only_path_that_can_call_af094(){
        let state=0x240000;let mut b=B::default();seed_base(&mut b,state,3);
        b.set32(state+0xa0,1);b.set8(state+0x90,0x80);b.set8(state+0x135,0);
        b.set8(state+0x0f,2);b.set8(state+0x9e,0);b.set32(STAGE116_OPTIONAL_215C20,1);
        b.q(STAGE116_CALL_3A604,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_25288,BtStage116Regs{r0:1,..Default::default()});
        b.q(STAGE116_CALL_2521C,BtStage116Regs{r0:1,..Default::default()});
        b.q(STAGE116_CALL_21FC2,BtStage116Regs{r0:1,..Default::default()});
        b.q(STAGE116_CALL_6304C,BtStage116Regs{r0:0,..Default::default()});
        let _=bt_stage116_register_state(state,3,0,0,&mut b);
        assert!(b.call_seen(STAGE116_CALL_AF094));
    }

    #[test]
    fn common_path_replaces_only_low16_of_arg1_scratch(){
        let state=0x250000;let arg1=0xABCD_0003;let mut b=B::default();seed_base(&mut b,state,arg1);
        b.set8(state+0x0f,2);
        b.set32(state+0xa0,0); // direct common
        b.set16(state+0x98,0x1234);
        b.q(STAGE116_CALL_3A604,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_25288,BtStage116Regs{r0:1,..Default::default()});
        b.q(STAGE116_CALL_2521C,BtStage116Regs{r0:1,..Default::default()});
        let _=bt_stage116_register_state(state,arg1,0,0,&mut b);
        let f=*b.final_inputs.last().unwrap();
        assert_eq!((f.0,f.1),(4,state));
        let scratch=f.2;
        assert_eq!(scratch>>16,0xABCD);
        assert_ne!(scratch&0xffff,arg1&0xffff);
    }

    #[test]
    fn tail_bit3_not_bit4_controls_629d0(){
        let state=0x260000;let mut b=B::default();seed_base(&mut b,state,3);
        b.set32(state+0xa0,0);b.set8(state+0x0f,2);
        b.set8(STAGE116_FLAGS_208338+0x13,0x08);
        b.q(STAGE116_CALL_3A604,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_25288,BtStage116Regs{r0:1,..Default::default()});
        b.q(STAGE116_CALL_2521C,BtStage116Regs{r0:1,..Default::default()});
        let _=bt_stage116_register_state(state,3,0,0,&mut b);
        assert!(b.call_seen(STAGE116_CALL_629D0));
    }

    #[test]
    fn stack_guard_failure_boundary_gets_saved_guard_in_r2_and_current_guard_in_r3(){
        let state=0x270000;let mut b=B::default();seed_base(&mut b,state,3);
        b.q(STAGE116_CALL_3A604,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_25288,BtStage116Regs{r0:0,..Default::default()});
        b.q(STAGE116_CALL_202E8,BtStage116Regs{r0:0x44,r1:0x11,r2:0x22,r3:0x33});
        b.q(STAGE116_CALL_94C0,BtStage116Regs{r0:0xDEAD,..Default::default()});
        // First read at entry saves A5A55A5A; a later guard read sees a mismatch.
        b.set32(STAGE116_STACK_GUARD_WORD,0x5A5A_A5A5);
        // Preserve the saved entry value by supplying it explicitly through a one-shot
        // read sequence is not supported by this byte-map mock, so call the exact finish
        // helper directly for the tuple oracle.
        let mut regs=BtStage116Regs{r0:0x44,r1:0x11,r2:0x22,r3:0x33};
        regs.r2=0; // overwritten by s116_finish
        let out=s116_finish(&mut b,0xA5A5_5A5A,regs);
        assert_eq!(out,0xDEAD);
        let call=b.calls.iter().rev().find(|x|x.0==STAGE116_CALL_94C0).unwrap().1;
        assert_eq!(call,BtStage116Regs{r0:0x44,r1:0x11,r2:0xA5A5_5A5A,r3:0x5A5A_A5A5});
    }

    #[test]
    fn provenance_and_literal_islands_are_exact(){
        assert_eq!(STAGE116_CURRENT_ADDR,0x16E0D8);
        assert_eq!(STAGE116_LEGACY_ADDR,0x16B10C);
        assert_eq!(STAGE116_BODY_LEN,1410);
        assert_eq!((STAGE116_INLINE_LITERAL_START,STAGE116_INLINE_LITERAL_END),(0x16E380,0x16E3A8));
        assert_eq!((STAGE116_POST_BODY_NOP,STAGE116_POST_BODY_LITERAL_START,STAGE116_POST_BODY_LITERAL_END),(0x16E65A,0x16E65C,0x16E6B0));
        assert_eq!(STAGE116_CALL_3A604,0x3A604);
        assert_eq!(STAGE116_CALL_3A742,0x3A742);
    }
}

pub const STAGE117_CURRENT_ADDR:u32=0x0016_E6B0;
pub const STAGE117_LEGACY_ADDR:u32=0x0016_B6E4;
pub const STAGE117_BODY_LEN:u32=862;
pub const STAGE117_INLINE_ISLAND_START:u32=0x0016_E8FE;
pub const STAGE117_INLINE_LITERAL_START:u32=0x0016_E900;
pub const STAGE117_INLINE_ISLAND_END:u32=0x0016_E91C;
pub const STAGE117_POST_BODY_NOP:u32=0x0016_EA0E;
pub const STAGE117_POST_BODY_LITERAL_START:u32=0x0016_EA10;
pub const STAGE117_POST_BODY_LITERAL_END:u32=0x0016_EA40;

pub const STAGE117_G_209BFC:u32=0x0020_9BFC;
pub const STAGE117_G_318B2C:u32=0x0031_8B2C;
pub const STAGE117_G_208338:u32=0x0020_8338;
pub const STAGE117_G_20AE7C:u32=0x0020_AE7C;
pub const STAGE117_G_206F78:u32=0x0020_6F78;
pub const STAGE117_G_207B84:u32=0x0020_7B84;
pub const STAGE117_G_209B94:u32=0x0020_9B94;
pub const STAGE117_G_207BA8:u32=0x0020_7BA8;
pub const STAGE117_G_209510:u32=0x0020_9510;
pub const STAGE117_G_20E929:u32=0x0020_E929;
pub const STAGE117_G_20E9FE:u32=0x0020_E9FE;
pub const STAGE117_G_20EAD1:u32=0x0020_EAD1;
pub const STAGE117_G_20B225:u32=0x0020_B225;
pub const STAGE117_G_20A229:u32=0x0020_A229;
pub const STAGE117_G_208B75:u32=0x0020_8B75;
pub const STAGE117_G_20B258:u32=0x0020_B258;
pub const STAGE117_G_20B2EB:u32=0x0020_B2EB;
pub const STAGE117_G_207FC1:u32=0x0020_7FC1;
pub const STAGE117_G_20B278:u32=0x0020_B278;

pub const STAGE117_CALL_335AC:u32=0x0003_35AC;
pub const STAGE117_CALL_4E108:u32=0x0004_E108;
pub const STAGE117_CALL_30F1C:u32=0x0003_0F1C;
pub const STAGE117_CALL_37FD0:u32=0x0003_7FD0;
pub const STAGE117_CALL_2529C:u32=0x0002_529C;
pub const STAGE117_CALL_1EF70:u32=0x0001_EF70;
pub const STAGE117_CALL_2A460:u32=0x0002_A460;
pub const STAGE117_CALL_29FF0:u32=0x0002_9FF0;
pub const STAGE117_CALL_1ECEC:u32=0x0001_ECEC;
pub const STAGE117_CALL_622AC:u32=0x0006_22AC;
pub const STAGE117_CALL_1E498:u32=0x0001_E498;
pub const STAGE117_CALL_1D104:u32=0x0001_D104;
pub const STAGE117_CALL_1E470:u32=0x0001_E470;
pub const STAGE117_CALL_63078:u32=0x0006_3078;
pub const STAGE117_CALL_2033C:u32=0x0002_033C;
pub const STAGE117_CALL_338FC:u32=0x0003_38FC;
pub const STAGE117_CALL_35C20:u32=0x0003_5C20;
pub const STAGE117_CALL_2C694:u32=0x0002_C694;
pub const STAGE117_CALL_428B0:u32=0x0004_28B0;
pub const STAGE117_CALL_3A742:u32=0x0003_A742;
pub const STAGE117_CALL_A9470:u32=0x000A_9470;
pub const STAGE117_CALL_A8090:u32=0x000A_8090;
pub const STAGE117_TAIL_2CAA8:u32=0x0002_CAA8;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage117Regs{pub r0:u32,pub r1:u32,pub r2:u32,pub r3:u32}

pub trait BtStage117Backend{
    fn read8(&mut self,addr:u32)->u8;
    fn read16(&mut self,addr:u32)->u16;
    fn read32(&mut self,addr:u32)->u32;
    fn write8(&mut self,addr:u32,value:u8);
    fn write16(&mut self,addr:u32,value:u16);
    fn write32(&mut self,addr:u32,value:u32);
    fn call(&mut self,target:u32,regs:BtStage117Regs)->BtStage117Regs;
    fn indirect_call(&mut self,target:u32,regs:BtStage117Regs)->BtStage117Regs;
    fn tail_call(&mut self,target:u32,regs:BtStage117Regs)->BtStage117Regs;
}

#[inline]
fn s117_bfi(dst:u32,src:u32,lsb:u32,width:u32)->u32{
    let low=(1u32<<width)-1; let mask=low<<lsb;
    (dst&!mask)|((src&low)<<lsb)
}
#[inline] fn s117_mode(v:u8)->u32{(u32::from(v)>>3)&0x0f}
#[inline]
fn s117_smlabb(a:u32,b:u32,acc:u32)->u32{
    let aa=(a as u16 as i16) as i32;
    let bb=(b as u16 as i16) as i32;
    aa.wrapping_mul(bb).wrapping_add(acc as i32) as u32
}

fn s117_pretransition_optional<B:BtStage117Backend>(
    state:u32,backend:&mut B,mut regs:BtStage117Regs
)->BtStage117Regs{
    regs=backend.call(STAGE117_CALL_2A460,regs);
    if regs.r0!=0{
        regs.r3=s117_mode(backend.read8(state.wrapping_add(0x90)));
        regs.r3=regs.r3.wrapping_add(0x0b)&0x0f;
        if regs.r3>2{
            regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
            regs=backend.call(STAGE117_CALL_29FF0,regs);
        }
    }
    regs
}

/// Exact current-HCD register/memory model for `0x16E6B0..0x16EA0E`.
///
/// The model keeps every current R0-R3 tuple across opaque direct/indirect calls,
/// treats `0x16E8FE..0x16E91C` as unreachable inline data, preserves fresh state/global
/// rereads and the signed-low-halfword SMLABB table indexing, and exposes the final
/// `B.W 0x2CAA8` as an explicit opaque tail boundary.
pub fn bt_stage117_register_state<B:BtStage117Backend>(
    state:u32,incoming_r1:u32,incoming_r2:u32,incoming_r3:u32,backend:&mut B
)->u32{
    let mut regs=BtStage117Regs{r0:state,r1:incoming_r1,r2:incoming_r2,r3:incoming_r3};

    regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
    regs=backend.call(STAGE117_CALL_335AC,regs);
    let saved_context=regs.r0; // callee-saved R6 until current 0x16E954.

    regs.r3=STAGE117_G_209BFC;
    regs.r3=backend.read32(regs.r3);
    if regs.r3!=0{
        regs.r0=state;
        regs=backend.call(STAGE117_CALL_4E108,regs);
    }

    regs.r3=u32::from(backend.read8(state.wrapping_add(0x94)));
    if regs.r3==0{
        regs.r3=backend.read32(state);
        regs.r0=state;
        regs.r3=backend.read32(regs.r3.wrapping_add(0x20));
        regs=backend.indirect_call(regs.r3,regs);
    }

    regs.r3=STAGE117_G_318B2C;
    let mut r7=backend.read32(state.wrapping_add(0xe8));
    let snapshot=backend.read32(regs.r3); // callee-saved R5 snapshot.

    if r7!=0{
        regs.r0=snapshot<<30;
        if snapshot&(1<<1)==0{
            regs.r1=0x29;
        }else{
            regs.r3=u32::from(backend.read8(state.wrapping_add(0x94)));
            if regs.r3!=2 || snapshot&(1<<18)==0{
                regs.r1=0x2a;
            }else{
                regs.r1=0;
                regs.r0=u32::from(backend.read8(state.wrapping_add(0x10)));
                regs=backend.call(STAGE117_CALL_30F1C,regs);
                regs.r1=regs.r0&0xff;
            }
        }
        regs.r0=r7;
        regs=backend.call(STAGE117_CALL_37FD0,regs);
    }

    regs=backend.call(STAGE117_CALL_2529C,regs);
    let mut skip_transition=false;
    if regs.r0==0{
        regs.r3=1;
        backend.write8(state.wrapping_add(0x95),1);
        regs.r3=u32::from(backend.read8(state.wrapping_add(0x99)));
        regs.r3=s117_bfi(regs.r3,regs.r0,0,1);
        backend.write8(state.wrapping_add(0x99),regs.r3 as u8);
        skip_transition=true;
    }else{
        regs.r3=u32::from(backend.read8(state.wrapping_add(0x94)));
        let mut enter_transition=true;
        let mut reach_pretransition=false;
        if regs.r3==2{
            reach_pretransition=true;
        }else{
            regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
            regs=backend.call(STAGE117_CALL_1EF70,regs);
            if regs.r0==1{ reach_pretransition=true; }
            else{ enter_transition=false; skip_transition=true; }
        }
        if reach_pretransition{
            regs.r3=u32::from(backend.read8(state.wrapping_add(0x0f)));
            if regs.r3==0{ regs=s117_pretransition_optional(state,backend,regs); }
        }

        if enter_transition{
            r7=state.wrapping_add(0x90);
            regs.r1=snapshot;
            regs.r0=r7;
            regs=backend.call(STAGE117_CALL_1ECEC,regs);

            regs.r3=u32::from(backend.read8(state.wrapping_add(0x90)));
            regs.r2=u32::from(backend.read8(state.wrapping_add(0x98)));
            regs.r1=regs.r3&7;
            regs.r2&=7;
            if regs.r1==regs.r2{
                regs.r2=STAGE117_G_208338;
                regs.r2=u32::from(backend.read8(regs.r2.wrapping_add(0x13)));
                regs.r2<<=27;
                let mut took_fast=false;
                if regs.r2&0x8000_0000!=0{
                    regs.r2=STAGE117_G_20AE7C;
                    regs.r2=backend.read32(regs.r2);
                    regs.r2=backend.read32(regs.r2);
                    regs.r3=s117_mode(regs.r3 as u8);
                    regs.r2=(regs.r2>>15)&0x0f;
                    if regs.r3==regs.r2{
                        regs.r1=snapshot;
                        regs.r0=state;
                        regs=backend.call(STAGE117_CALL_622AC,regs);
                        took_fast=true;
                    }
                }
                if !took_fast{
                    regs.r3=u32::from(backend.read16(state.wrapping_add(0x92)));
                    regs.r3 &= !(0x03ff<<3);
                    backend.write16(state.wrapping_add(0x92),regs.r3 as u16);
                    regs.r2=snapshot;
                    regs.r3=0;
                    regs.r1=state.wrapping_add(0x28);
                    regs.r0=r7;
                    regs=backend.call(STAGE117_CALL_1E498,regs);

                    regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
                    regs=backend.call(STAGE117_CALL_1EF70,regs);
                    if regs.r0==1{
                        regs.r3=u32::from(backend.read8(state.wrapping_add(0x94)));
                        if regs.r3==1{
                            regs.r3=u32::from(backend.read8(state.wrapping_add(0x99)));
                            regs.r3&=!1;
                            backend.write8(state.wrapping_add(0x99),regs.r3 as u8);
                            regs.r3=u32::from(backend.read8(state.wrapping_add(0x90)));
                            regs.r3|=0x80;
                            backend.write8(state.wrapping_add(0x90),regs.r3 as u8);
                        }
                    }

                    let state95=backend.read8(state.wrapping_add(0x95));
                    regs.r3=u32::from(state95);
                    regs.r3=STAGE117_G_206F78;
                    let mut do_notify=false;
                    if state95!=1{
                        regs.r3=u32::from(backend.read8(regs.r3.wrapping_add(4)));
                        regs.r2=1;
                        backend.write8(state.wrapping_add(0x127),1);
                        if regs.r3!=0{
                            regs.r3=u32::from(backend.read8(state.wrapping_add(0x90)));
                            regs.r1=u32::from(backend.read16(state.wrapping_add(0x90)));
                            let mode=s117_mode(regs.r3 as u8);
                            if mode>1{ regs.r3=u32::from(backend.read16(state.wrapping_add(0x92))); }
                            regs.r1=(regs.r1>>3)&0x7f;
                            if mode<=1{ regs.r0=2; }
                            else{ regs.r1|=regs.r3<<7; regs.r0=4; }
                            do_notify=true;
                        }
                    }else{
                        regs.r3=u32::from(backend.read8(regs.r3.wrapping_add(4)));
                        if regs.r3!=0{
                            regs.r2=STAGE117_G_207B84;
                            regs.r3=u32::from(backend.read16(state.wrapping_add(0x90)));
                            regs.r1=u32::from(backend.read8(regs.r2));
                            regs.r3=(regs.r3>>3)&0x7f;
                            regs.r1=regs.r3|(regs.r1<<7);
                            regs.r0=5;
                            do_notify=true;
                        }
                    }
                    if do_notify{ regs=backend.call(STAGE117_CALL_1D104,regs); }
                    regs.r3=u32::from(backend.read8(state.wrapping_add(0x99)));
                    regs.r0=regs.r3<<31;
                    if regs.r0&0x8000_0000!=0{
                        regs.r3=0;
                        backend.write8(state.wrapping_add(0x11c),0);
                    }
                }
            }else{
                regs.r3&=7;
                if regs.r3==0{
                    regs.r2=u32::from(backend.read8(state.wrapping_add(0x99)));
                    regs.r2=s117_bfi(regs.r2,regs.r3,0,1);
                    regs.r3=snapshot&0x003c_0000;
                    let eq=regs.r3==0x0004_0000;
                    backend.write8(state.wrapping_add(0x99),regs.r2 as u8);
                    if eq{
                        regs.r3=2;
                        backend.write8(state.wrapping_add(0x95),2);
                        regs.r1=snapshot;
                        regs.r0=r7;
                        regs=backend.call(STAGE117_CALL_1E470,regs);
                    }
                }
            }
        }
    }

    let _=skip_transition; // Documents the direct branch to current 0x16E87A.
    regs.r0=state;
    regs=backend.call(STAGE117_CALL_63078,regs);

    regs.r3=u32::from(backend.read8(state.wrapping_add(0x0f)));
    if regs.r3==1{
        #[derive(Clone,Copy)] enum Pc{P886,P898,P88E,P8A4,P8AA,P8C6,P8BC,P91C,Done}
        let mut pc=Pc::P886;
        loop{
            pc=match pc{
                Pc::P886=>{
                    regs.r3=u32::from(backend.read8(state.wrapping_add(0x94)));
                    if regs.r3==1{Pc::P898}else{Pc::P88E}
                }
                Pc::P898=>{
                    regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
                    regs=backend.call(STAGE117_CALL_1EF70,regs);
                    if regs.r0==1{Pc::P88E}else{Pc::P8A4}
                }
                Pc::P88E=>{
                    regs.r3=u32::from(backend.read8(state.wrapping_add(0x94)));
                    if regs.r3==2{Pc::P8AA}else{Pc::P8BC}
                }
                Pc::P8A4=>{regs=backend.call(STAGE117_CALL_2033C,regs);Pc::Done}
                Pc::P8AA=>{
                    regs.r3=u32::from(backend.read8(state.wrapping_add(0x90)));
                    regs.r2=regs.r3&0x78;
                    if regs.r2!=0{Pc::P8C6}else{
                        regs.r3=backend.read32(state.wrapping_add(0xa0));
                        if regs.r3==0{Pc::P8A4}else{Pc::P8BC}
                    }
                }
                Pc::P8C6=>{
                    if regs.r2!=8{Pc::P8BC}else{
                        regs.r2=backend.read32(state.wrapping_add(0xa0));
                        if regs.r2!=0{Pc::P8BC}else{
                            regs.r2=STAGE117_G_209B94;
                            regs.r1=backend.read32(state.wrapping_add(0xf8));
                            regs.r2=backend.read32(regs.r2);
                            if regs.r1&regs.r2==0{Pc::P8BC}else{
                                regs.r2=u32::from(backend.read8(state.wrapping_add(0x117)));
                                if regs.r2==0{Pc::P8BC}else{
                                    regs.r1=regs.r3<<24;
                                    if regs.r1&0x8000_0000==0{Pc::P8BC}else{
                                        regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
                                        regs=backend.call(STAGE117_CALL_338FC,regs);
                                        if regs.r0==0{Pc::P8BC}else{
                                            regs.r3=u32::from(backend.read8(regs.r0.wrapping_add(0x1b)));
                                            regs.r2=regs.r3<<31;
                                            if regs.r2&0x8000_0000==0{Pc::P8A4}else{Pc::P8BC}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Pc::P8BC=>{
                    regs.r3=u32::from(backend.read8(state.wrapping_add(0x94)));
                    if regs.r3==2{Pc::P91C}else{Pc::Done}
                }
                Pc::P91C=>{
                    regs.r3=u32::from(backend.read8(state.wrapping_add(0x90)));
                    regs.r3<<=29;
                    if regs.r3==0{Pc::P8A4}else{Pc::Done}
                }
                Pc::Done=>break,
            };
        }
    }

    regs.r3=u32::from(backend.read8(state.wrapping_add(0x97)));
    if regs.r3==0{
        regs.r3=1;
        backend.write8(state.wrapping_add(0x97),1);
        regs.r0=state;
        regs=backend.call(STAGE117_CALL_35C20,regs);
    }

    regs.r3=STAGE117_G_207BA8;
    regs.r3=u32::from(backend.read8(regs.r3));
    if regs.r3!=0 && saved_context!=0{
        regs.r3=u32::from(backend.read8(saved_context.wrapping_add(0x168)));
        if regs.r3!=0{
            regs.r1=1;
            regs.r0=state;
            regs=backend.call(STAGE117_CALL_2C694,regs);
        }
    }

    let table_ptr_addr=STAGE117_G_209510;
    regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
    regs.r2=backend.read32(table_ptr_addr);
    let stride=0x78u32;
    let minus_stride=!0x77u32;
    regs.r3=s117_smlabb(stride,regs.r0,minus_stride);
    regs.r3=regs.r3.wrapping_add(regs.r2);
    regs.r2=u32::from(backend.read8(regs.r3.wrapping_add(1)));
    if regs.r2!=0{
        regs.r3=u32::from(backend.read8(regs.r3.wrapping_add(2)));
        if regs.r3!=0{
            regs.r1=0;
            regs.r0=regs.r0.wrapping_sub(1);
            regs=backend.call(STAGE117_CALL_428B0,regs);
            regs.r3=u32::from(backend.read8(state.wrapping_add(0xa4)));
            let mut entry=s117_smlabb(stride,regs.r3,minus_stride);
            regs.r3=backend.read32(table_ptr_addr); // true post-call reread
            entry=entry.wrapping_add(regs.r3);
            regs.r3=0;
            backend.write8(entry.wrapping_add(2),0);
        }
    }

    regs.r2=0;
    regs.r1=state;
    regs.r0=6;
    regs=backend.call(STAGE117_CALL_3A742,regs);

    regs.r3=u32::from(backend.read8(state.wrapping_add(0x121)));
    if regs.r3==1{
        regs.r3=STAGE117_G_20E929;
        regs.r2=u32::from(backend.read8(state.wrapping_add(0xa4)));
        backend.write8(regs.r3,regs.r2 as u8);

        regs.r3=STAGE117_G_20E9FE;
        regs.r3=u32::from(backend.read8(regs.r3));
        let mut role_gate=false;
        if regs.r3==1{
            regs.r3=STAGE117_G_20EAD1;
            regs.r3=u32::from(backend.read8(regs.r3));
            if regs.r3==1{role_gate=true;}
        }
        if !role_gate{
            regs.r3=STAGE117_G_20B225;
            regs.r3=u32::from(backend.read8(regs.r3));
            if regs.r3!=0{role_gate=true;}
        }
        if role_gate{
            regs.r3=u32::from(backend.read8(state.wrapping_add(0x0f)));
            let mut call_a9470=false;
            if regs.r3==0{
                call_a9470=true;
            }else if regs.r3==1{
                regs.r3=STAGE117_G_20A229;
                regs.r3=u32::from(backend.read8(regs.r3));
                if regs.r3!=0{
                    regs.r3=snapshot&1;
                    regs.r2=STAGE117_G_208B75;
                    if regs.r3!=0{
                        regs.r3=u32::from(backend.read8(regs.r2)).wrapping_add(1);
                        backend.write8(regs.r2,regs.r3 as u8);
                        regs.r3&=0xff;
                        if regs.r3>0x14{call_a9470=true;}
                    }else{
                        backend.write8(regs.r2,regs.r3 as u8);
                        call_a9470=true;
                    }
                }
            }
            if call_a9470{
                regs.r1=snapshot;
                regs.r0=state;
                regs=backend.call(STAGE117_CALL_A9470,regs);
            }
        }
    }

    regs.r3=STAGE117_G_20B258;
    regs.r3=u32::from(backend.read8(regs.r3));
    let mut call_a8090=regs.r3!=0;
    if !call_a8090{
        regs.r3=STAGE117_G_20B2EB;
        regs.r3=u32::from(backend.read8(regs.r3));
        call_a8090=regs.r3!=0;
    }
    if call_a8090{
        regs.r1=snapshot;
        regs.r0=state;
        regs=backend.call(STAGE117_CALL_A8090,regs);
    }

    regs.r3=STAGE117_G_207FC1;
    regs.r3=u32::from(backend.read8(regs.r3));
    if regs.r3!=0{
        regs.r3=STAGE117_G_20B278;
        regs.r3=u32::from(backend.read8(regs.r3));
        if regs.r3!=0{
            regs.r0=state;
            regs.r1=0x10;
            regs=backend.tail_call(STAGE117_TAIL_2CAA8,regs);
        }
    }
    regs.r0
}

#[cfg(test)]
mod stage117_tests{
    use super::*;
    use std::collections::{BTreeMap,VecDeque};
    use std::vec::Vec;
    use std::vec;

    #[derive(Default)]
    struct B{
        mem:BTreeMap<u32,u8>,
        returns:BTreeMap<u32,VecDeque<BtStage117Regs>>,
        calls:Vec<(u32,BtStage117Regs)>,
        indirect:Vec<(u32,BtStage117Regs)>,
        tails:Vec<(u32,BtStage117Regs)>,
        mutate:BTreeMap<u32,Vec<(u32,u8)>>,
    }
    impl B{
        fn set8(&mut self,a:u32,v:u8){self.mem.insert(a,v);}
        fn set16(&mut self,a:u32,v:u16){for(i,b)in v.to_le_bytes().iter().enumerate(){self.set8(a+i as u32,*b)}}
        fn set32(&mut self,a:u32,v:u32){for(i,b)in v.to_le_bytes().iter().enumerate(){self.set8(a+i as u32,*b)}}
        fn q(&mut self,t:u32,v:BtStage117Regs){self.returns.entry(t).or_default().push_back(v);}
        fn seen(&self,t:u32)->bool{self.calls.iter().any(|x|x.0==t)}
        fn apply_mut(&mut self,t:u32){if let Some(v)=self.mutate.remove(&t){for(a,b)in v{self.set8(a,b)}}}
    }
    impl BtStage117Backend for B{
        fn read8(&mut self,a:u32)->u8{*self.mem.get(&a).unwrap_or(&0)}
        fn read16(&mut self,a:u32)->u16{u16::from_le_bytes([self.read8(a),self.read8(a+1)])}
        fn read32(&mut self,a:u32)->u32{u32::from_le_bytes([self.read8(a),self.read8(a+1),self.read8(a+2),self.read8(a+3)])}
        fn write8(&mut self,a:u32,v:u8){self.mem.insert(a,v);}
        fn write16(&mut self,a:u32,v:u16){self.set16(a,v)}
        fn write32(&mut self,a:u32,v:u32){self.set32(a,v)}
        fn call(&mut self,t:u32,r:BtStage117Regs)->BtStage117Regs{
            self.calls.push((t,r)); self.apply_mut(t);
            self.returns.get_mut(&t).and_then(|q|q.pop_front()).unwrap_or(r)
        }
        fn indirect_call(&mut self,t:u32,r:BtStage117Regs)->BtStage117Regs{
            self.indirect.push((t,r));
            self.returns.get_mut(&t).and_then(|q|q.pop_front()).unwrap_or(r)
        }
        fn tail_call(&mut self,t:u32,r:BtStage117Regs)->BtStage117Regs{
            self.tails.push((t,r));
            self.returns.get_mut(&t).and_then(|q|q.pop_front()).unwrap_or(r)
        }
    }

    fn seed(b:&mut B,state:u32){
        b.set8(state+0xa4,2);
        b.set8(state+0x94,3); // skip indirect and default role-mode fast paths.
        b.set8(state+0x0f,2);
        b.set8(state+0x97,1);
        b.set32(state+0xe8,0);
        b.set32(STAGE117_G_209BFC,0);
        b.set32(STAGE117_G_318B2C,0);
        b.set32(STAGE117_G_209510,0);
        b.set8(STAGE117_G_207BA8,0);
    }

    #[test]
    fn entry_preserves_tuple_and_zero_state94_calls_vtable_slot20_indirectly(){
        let state=0x200000;let mut b=B::default();seed(&mut b,state);
        b.set8(state+0x94,0);b.set32(state,0x5000);b.set32(0x5020,0x7777);b.set32(STAGE117_G_209BFC,1);
        b.q(STAGE117_CALL_335AC,BtStage117Regs{r0:0x6000,r1:0x11,r2:0x22,r3:0x33});
        b.q(STAGE117_CALL_4E108,BtStage117Regs{r0:9,r1:0x21,r2:0x22,r3:0x23});
        let _=bt_stage117_register_state(state,1,2,3,&mut b);
        assert_eq!(b.calls[0],(STAGE117_CALL_335AC,BtStage117Regs{r0:2,r1:1,r2:2,r3:3}));
        assert_eq!(b.indirect[0],(0x7777,BtStage117Regs{r0:state,r1:0x21,r2:0x22,r3:0x7777}));
    }

    #[test]
    fn nonzero_e8_uses_custom_reason_from_30f1c_when_snapshot_bits_one_and_eighteen_set(){
        let state=0x210000;let mut b=B::default();seed(&mut b,state);
        b.set32(state+0xe8,0x9000);b.set32(STAGE117_G_318B2C,(1<<1)|(1<<18));b.set8(state+0x94,2);b.set8(state+0x10,7);
        b.q(STAGE117_CALL_30F1C,BtStage117Regs{r0:0x1234,r1:8,r2:9,r3:10});
        let _=bt_stage117_register_state(state,0,0,0,&mut b);
        let c=b.calls.iter().find(|x|x.0==STAGE117_CALL_37FD0).unwrap().1;
        assert_eq!((c.r0,c.r1),(0x9000,0x34));
    }

    #[test]
    fn zero_2529c_sets_95_and_clears_only_bit0_of_99(){
        let state=0x220000;let mut b=B::default();seed(&mut b,state);b.set8(state+0x99,0xff);
        b.q(STAGE117_CALL_2529C,BtStage117Regs{r0:0,r1:0x11,r2:0x22,r3:0x33});
        let _=bt_stage117_register_state(state,0,0,0,&mut b);
        assert_eq!(b.read8(state+0x95),1);assert_eq!(b.read8(state+0x99),0xfe);assert!(b.seen(STAGE117_CALL_63078));
    }

    #[test]
    fn equal_low3_and_global_mode_gate_takes_622ac_with_exact_tuple(){
        let state=0x230000;let mut b=B::default();seed(&mut b,state);
        let snap=0x11223344;b.set32(STAGE117_G_318B2C,snap);b.set8(state+0x94,2);b.set8(state+0x0f,1);
        b.set8(state+0x90,0x18);b.set8(state+0x98,0);b.set8(STAGE117_G_208338+0x13,0x10);
        b.set32(STAGE117_G_20AE7C,0x5000);b.set32(0x5000,3<<15);
        b.q(STAGE117_CALL_2529C,BtStage117Regs{r0:1,..Default::default()});
        let _=bt_stage117_register_state(state,0,0,0,&mut b);
        let c=b.calls.iter().find(|x|x.0==STAGE117_CALL_622AC).unwrap().1;
        assert_eq!(c,BtStage117Regs{r0:state,r1:snap,r2:3,r3:3});
        assert!(!b.seen(STAGE117_CALL_1E498));
    }

    #[test]
    fn unequal_low3_zero_with_snapshot_selector_sets_95_two_and_calls_1e470(){
        let state=0x240000;let mut b=B::default();seed(&mut b,state);
        let snap=0x0004_0000;b.set32(STAGE117_G_318B2C,snap);b.set8(state+0x94,2);b.set8(state+0x0f,1);
        b.set8(state+0x90,0x00);b.set8(state+0x98,1);b.set8(state+0x99,0xff);
        b.q(STAGE117_CALL_2529C,BtStage117Regs{r0:1,..Default::default()});
        let _=bt_stage117_register_state(state,0,0,0,&mut b);
        assert_eq!(b.read8(state+0x95),2);assert_eq!(b.read8(state+0x99),0xfe);
        let c=b.calls.iter().find(|x|x.0==STAGE117_CALL_1E470).unwrap().1;
        assert_eq!((c.r0,c.r1),(state+0x90,snap));
    }

    #[test]
    fn role_one_gate_can_reach_2033c_when_lookup_byte1b_bit0_is_clear(){
        let state=0x250000;let mut b=B::default();seed(&mut b,state);
        b.set8(state+0x0f,1);b.set8(state+0x94,2);b.set8(state+0x90,0x88);b.set32(state+0xa0,0);
        b.set32(state+0xf8,1);b.set32(STAGE117_G_209B94,1);b.set8(state+0x117,1);
        b.q(STAGE117_CALL_2529C,BtStage117Regs{r0:0,..Default::default()});
        b.q(STAGE117_CALL_338FC,BtStage117Regs{r0:0x7000,..Default::default()});b.set8(0x701b,0);
        let _=bt_stage117_register_state(state,0,0,0,&mut b);
        assert!(b.seen(STAGE117_CALL_338FC));assert!(b.seen(STAGE117_CALL_2033C));
    }

    #[test]
    fn smlabb_table_path_rereads_base_after_428b0_and_clears_new_entry_byte2(){
        let state=0x260000;let mut b=B::default();seed(&mut b,state);b.set8(state+0xa4,2);
        b.set32(STAGE117_G_209510,0x1000);b.set8(0x1000+0x78+1,1);b.set8(0x1000+0x78+2,1);
        b.set8(0x2000+0x78+2,9);b.mutate.insert(STAGE117_CALL_428B0,vec![(STAGE117_G_209510,0x00),(STAGE117_G_209510+1,0x20),(STAGE117_G_209510+2,0),(STAGE117_G_209510+3,0)]);
        let _=bt_stage117_register_state(state,0,0,0,&mut b);
        let c=b.calls.iter().find(|x|x.0==STAGE117_CALL_428B0).unwrap().1;
        assert_eq!((c.r0,c.r1),(1,0));assert_eq!(b.read8(0x2000+0x78+2),0);
    }

    #[test]
    fn role_one_counter_over_twenty_calls_a9470_and_uses_snapshot(){
        let state=0x270000;let mut b=B::default();seed(&mut b,state);let snap=1;b.set32(STAGE117_G_318B2C,snap);
        b.set8(state+0x121,1);b.set8(STAGE117_G_20B225,1);b.set8(state+0x0f,1);b.set8(STAGE117_G_20A229,1);b.set8(STAGE117_G_208B75,20);
        let _=bt_stage117_register_state(state,0,0,0,&mut b);
        assert_eq!(b.read8(STAGE117_G_208B75),21);
        let c=b.calls.iter().find(|x|x.0==STAGE117_CALL_A9470).unwrap().1;assert_eq!((c.r0,c.r1),(state,snap));
    }

    #[test]
    fn final_two_global_gate_tails_2caa8_with_live_r2_and_fresh_r3(){
        let state=0x280000;let mut b=B::default();seed(&mut b,state);b.set8(STAGE117_G_207FC1,1);b.set8(STAGE117_G_20B278,1);
        b.q(STAGE117_CALL_3A742,BtStage117Regs{r0:0x44,r1:0x11,r2:0x22,r3:0x33});
        b.q(STAGE117_TAIL_2CAA8,BtStage117Regs{r0:0xdead,..Default::default()});
        assert_eq!(bt_stage117_register_state(state,0,0,0,&mut b),0xdead);
        let c=b.tails[0];assert_eq!(c.0,STAGE117_TAIL_2CAA8);assert_eq!(c.1,BtStage117Regs{r0:state,r1:0x10,r2:0x22,r3:1});
    }

    #[test]
    fn provenance_literal_islands_and_signed_stride_are_exact(){
        assert_eq!(STAGE117_CURRENT_ADDR,0x16E6B0);assert_eq!(STAGE117_LEGACY_ADDR,0x16B6E4);assert_eq!(STAGE117_BODY_LEN,862);
        assert_eq!((STAGE117_INLINE_ISLAND_START,STAGE117_INLINE_LITERAL_START,STAGE117_INLINE_ISLAND_END),(0x16E8FE,0x16E900,0x16E91C));
        assert_eq!((STAGE117_POST_BODY_NOP,STAGE117_POST_BODY_LITERAL_START,STAGE117_POST_BODY_LITERAL_END),(0x16EA0E,0x16EA10,0x16EA40));
        assert_eq!(s117_smlabb(0x78,0,!0x77),0xffff_ff88);assert_eq!(s117_smlabb(0x78,2,!0x77),0x78);
    }
}

pub const STAGE118_CURRENT_ADDR:u32=0x0016_EA40;
pub const STAGE118_LEGACY_ADDR:u32=0x0016_BA74;
pub const STAGE118_BODY_LEN:u32=344;
pub const STAGE118_CURRENT_RAW_SHA256:&str="d4e9f434c4cd15a8428b46635774db082b515ff093255eb3d56cc5cf896e7eb0";
pub const STAGE118_NORMALIZED_SHA256:&str="16631247ae727cd9f13e340418460b9d45c9a67de2d4ea1cbbe179f263e3ce3d";
pub const STAGE118_TRANSFER_OFFSETS:[u16;19]=[
    0x0C,0x18,0x1E,0x24,0x3A,0x94,0xBC,0xEE,0xF4,0xFE,
    0x10C,0x114,0x11C,0x122,0x126,0x12E,0x136,0x13E,0x150
];

pub const STAGE118_CALL_35060:u32=0x0003_5060;
pub const STAGE118_CALL_B082C:u32=0x000B_082C;
pub const STAGE118_CALL_19754:u32=0x0001_9754;
pub const STAGE118_CALL_780:u32=0x0000_0780;
pub const STAGE118_CALL_3DB4:u32=0x0000_3DB4;
pub const STAGE118_CALL_33D28:u32=0x0003_3D28;
pub const STAGE118_CALL_17DB8:u32=0x0001_7DB8;
pub const STAGE118_CALL_21F00:u32=0x0002_1F00;
pub const STAGE118_CALL_28A0C:u32=0x0002_8A0C;
pub const STAGE118_CALL_202C0:u32=0x0002_02C0;
pub const STAGE118_TAIL_1FB24:u32=0x0001_FB24;
pub const STAGE118_TAIL_202E8:u32=0x0002_02E8;
pub const STAGE118_CALL_19318:u32=0x0001_9318;
pub const STAGE118_CALL_21F42:u32=0x0002_1F42;
pub const STAGE118_CALL_21F0C:u32=0x0002_1F0C;
pub const STAGE118_TAIL_221AC:u32=0x0002_21AC;

pub const STAGE118_DIRECT_TRANSFERS:[u32;19]=[
    STAGE118_CALL_35060,
    STAGE118_CALL_B082C,
    STAGE118_CALL_19754,
    STAGE118_CALL_780,
    STAGE118_CALL_3DB4,
    STAGE118_CALL_33D28,
    STAGE118_CALL_17DB8,
    STAGE118_CALL_B082C,
    STAGE118_CALL_21F00,
    STAGE118_CALL_28A0C,
    STAGE118_CALL_202C0,
    STAGE118_TAIL_1FB24,
    STAGE118_TAIL_202E8,
    STAGE118_CALL_780,
    STAGE118_CALL_19318,
    STAGE118_CALL_28A0C,
    STAGE118_CALL_21F42,
    STAGE118_CALL_21F0C,
    STAGE118_TAIL_221AC,
];

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage118Regs{pub r0:u32,pub r1:u32,pub r2:u32,pub r3:u32}

pub trait BtStage118Backend{
    fn read8(&mut self,addr:u32)->u8;
    fn read32(&mut self,addr:u32)->u32;
    fn write8(&mut self,addr:u32,value:u8);
    fn write32(&mut self,addr:u32,value:u32);
    fn call(&mut self,target:u32,regs:BtStage118Regs)->BtStage118Regs;
    fn tail_call(&mut self,target:u32,regs:BtStage118Regs)->BtStage118Regs;
}

/// Exact current-HCD register/memory model for `0x16EA40..0x16EB98`.
///
/// The model preserves R0-R3 caller-volatiles through every opaque boundary,
/// keeps the three incoming callee-saved values used by the body, performs
/// each firmware reread separately, uses wrapping pointer arithmetic, and
/// exposes all three reachable B.W exits as opaque tail boundaries.
pub fn bt_stage118_register_state<B:BtStage118Backend>(
    incoming_r0:u32,incoming_r1:u32,incoming_r2:u32,incoming_r3:u32,backend:&mut B
)->u32{
    let saved_r5=incoming_r0;
    let saved_r7=incoming_r1;
    let saved_r6=incoming_r2;
    let mut regs=BtStage118Regs{
        r0:backend.read32(saved_r5),
        r1:incoming_r1,
        r2:incoming_r2,
        r3:incoming_r3,
    };

    regs=backend.call(STAGE118_CALL_35060,regs);
    let r4=regs.r0;
    if regs.r0==0{return regs.r0;}

    regs=backend.call(STAGE118_CALL_B082C,regs);
    let r8;
    if regs.r0==0{
        regs=backend.call(STAGE118_CALL_19754,regs);
        regs.r0=1;
        regs=backend.call(STAGE118_CALL_780,regs);
        r8=regs.r0;
    }else{
        r8=0;
    }

    if saved_r7!=0{
        regs.r2=0x10;
        regs.r1=saved_r7;
        regs.r0=r4.wrapping_add(0x2c);
        regs=backend.call(STAGE118_CALL_3DB4,regs);
    }

    regs.r2=u32::from(backend.read8(saved_r5.wrapping_add(0x57)));
    regs.r1=(regs.r2>>3)&1;
    regs.r3=(regs.r2>>2)&1;
    let pair=regs.r2&0x14;
    let pair_eq=pair==0x14;
    regs.r3=(regs.r3<<3)|(regs.r1<<2);
    regs.r2=if pair_eq{0x10}else{0};
    regs.r3|=regs.r2;
    backend.write32(r4.wrapping_add(0x28),regs.r3);

    if saved_r7!=0{
        if saved_r6!=0{
            regs.r3=backend.read32(saved_r6);
            backend.write32(r4.wrapping_add(0x3c),regs.r3);
            regs.r3=backend.read32(saved_r6.wrapping_add(4));
            backend.write32(r4.wrapping_add(0x40),regs.r3);
        }else{
            regs.r3=backend.read32(saved_r5.wrapping_add(0x130));
            backend.write32(r4.wrapping_add(0x3c),regs.r3);
            regs.r3=backend.read32(saved_r5.wrapping_add(0x134));
            backend.write32(r4.wrapping_add(0x40),regs.r3);
        }
    }

    regs.r3=(u32::from(backend.read8(saved_r5.wrapping_add(0x57)))>>3)&1;
    backend.write8(r4.wrapping_add(0x60),regs.r3 as u8);
    regs.r3=(u32::from(backend.read8(saved_r5.wrapping_add(0x57)))>>2)&1;
    backend.write8(r4.wrapping_add(0x5f),regs.r3 as u8);

    regs.r0=saved_r5;
    regs=backend.call(STAGE118_CALL_33D28,regs);
    if regs.r0!=0{
        regs.r3=backend.read32(r4.wrapping_add(0x28));
        regs.r3&=!0x10;
        backend.write32(r4.wrapping_add(0x28),regs.r3);

        if saved_r7!=0{
            regs.r2=0;
            regs.r3=0;
            let r10=backend.read32(r4.wrapping_add(0x50));
            let r11=backend.read32(r4.wrapping_add(0x54));
            backend.write32(r4.wrapping_add(0x48),regs.r2);
            backend.write32(r4.wrapping_add(0x4c),regs.r3);
            backend.write32(r4.wrapping_add(0x50),regs.r2);
            backend.write32(r4.wrapping_add(0x54),regs.r3);
            backend.write8(r4.wrapping_add(0x65),0);

            regs.r0=u32::from(backend.read8(r4.wrapping_add(0x0e)));
            regs=backend.call(STAGE118_CALL_17DB8,regs);
            backend.write8(r4.wrapping_add(0x64),0);
            backend.write32(r4.wrapping_add(0x58),regs.r0);

            if saved_r6==0{
                regs.r3=r10|r11;
                if regs.r3==0{
                    backend.write8(r4.wrapping_add(0x66),saved_r6 as u8);
                }
            }
        }

        regs.r3=backend.read32(r4.wrapping_add(0x28));
        if regs.r3!=0{
            regs.r3=1;
            backend.write8(r4.wrapping_add(0x5e),regs.r3 as u8);
            regs.r3=0;
            backend.write8(r4.wrapping_add(0x63),regs.r3 as u8);
        }else{
            backend.write8(r4.wrapping_add(0x5e),regs.r3 as u8);
        }
    }else{
        backend.write8(r4.wrapping_add(0x5e),regs.r0 as u8);
    }

    regs=backend.call(STAGE118_CALL_B082C,regs);
    if regs.r0!=0{
        regs=backend.call(STAGE118_CALL_21F00,regs);
        let saved_21f00_r0=regs.r0;
        regs.r0=u32::from(backend.read8(r4.wrapping_add(0xa4)));
        regs=backend.call(STAGE118_CALL_28A0C,regs);

        if r4==saved_21f00_r0 || (regs.r0!=0 && saved_21f00_r0==regs.r0){
            regs=backend.call(STAGE118_CALL_202C0,regs);
            return backend.tail_call(STAGE118_TAIL_1FB24,regs).r0;
        }
        return backend.tail_call(STAGE118_TAIL_202E8,regs).r0;
    }

    regs.r0=r8;
    regs=backend.call(STAGE118_CALL_780,regs);
    regs=backend.call(STAGE118_CALL_19318,regs);
    regs.r0=u32::from(backend.read8(r4.wrapping_add(0xa4)));
    regs=backend.call(STAGE118_CALL_28A0C,regs);
    let saved_28a0c_r0=regs.r0;
    if regs.r0!=0{
        regs=backend.call(STAGE118_CALL_21F42,regs);
        if regs.r0!=0{
            regs.r0=saved_28a0c_r0;
            regs=backend.call(STAGE118_CALL_21F0C,regs);
            if regs.r0==0x10{
                regs.r0=saved_28a0c_r0;
            }else{
                regs.r0=r4;
            }
        }else{
            regs.r0=r4;
        }
    }else{
        regs.r0=r4;
    }
    backend.tail_call(STAGE118_TAIL_221AC,regs).r0
}

#[cfg(test)]
mod stage118_tests{
    use super::*;
    use std::vec;
    use std::vec::Vec;
    use std::collections::{BTreeMap,BTreeSet,VecDeque};

    #[derive(Default)]
    struct B{
        mem:BTreeMap<u32,u8>,
        read8q:BTreeMap<u32,VecDeque<u8>>,
        ret:BTreeMap<u32,VecDeque<BtStage118Regs>>,
        calls:Vec<(u32,BtStage118Regs)>,
        tails:Vec<(u32,BtStage118Regs)>,
        writes8:Vec<(u32,u8)>,
        writes32:Vec<(u32,u32)>,
    }
    impl B{
        fn set8(&mut self,a:u32,v:u8){self.mem.insert(a,v);}
        fn set32(&mut self,a:u32,v:u32){for i in 0..4{self.set8(a.wrapping_add(i),((v>>(8*i))&0xff) as u8);}}
        fn get8(&self,a:u32)->u8{*self.mem.get(&a).unwrap_or(&0)}
        fn get32(&self,a:u32)->u32{
            (0..4).fold(0u32,|v,i|v|(u32::from(self.get8(a.wrapping_add(i)))<<(8*i)))
        }
        fn q(&mut self,t:u32,r:BtStage118Regs){self.ret.entry(t).or_default().push_back(r);}
        fn q8(&mut self,a:u32,vals:&[u8]){
            let q=self.read8q.entry(a).or_default();
            for &v in vals{q.push_back(v);}
        }
        fn nth(&self,t:u32,n:usize)->BtStage118Regs{
            self.calls.iter().filter(|x|x.0==t).nth(n).unwrap().1
        }
        fn seen(&self,t:u32)->bool{self.calls.iter().any(|x|x.0==t)}
    }
    impl BtStage118Backend for B{
        fn read8(&mut self,a:u32)->u8{
            if let Some(q)=self.read8q.get_mut(&a){
                if let Some(v)=q.pop_front(){return v;}
            }
            self.get8(a)
        }
        fn read32(&mut self,a:u32)->u32{self.get32(a)}
        fn write8(&mut self,a:u32,v:u8){self.writes8.push((a,v));self.set8(a,v);}
        fn write32(&mut self,a:u32,v:u32){self.writes32.push((a,v));self.set32(a,v);}
        fn call(&mut self,t:u32,r:BtStage118Regs)->BtStage118Regs{
            self.calls.push((t,r));
            self.ret.get_mut(&t).and_then(|q|q.pop_front()).unwrap_or(r)
        }
        fn tail_call(&mut self,t:u32,r:BtStage118Regs)->BtStage118Regs{
            self.tails.push((t,r));
            self.ret.get_mut(&t).and_then(|q|q.pop_front()).unwrap_or(r)
        }
    }

    fn seed(b:&mut B,state:u32,obj:u32){
        b.set32(state,0x1234_5678);
        b.set8(state+0x57,0);
        b.set8(state+0x130,0x44); b.set8(state+0x131,0x33); b.set8(state+0x132,0x22); b.set8(state+0x133,0x11);
        b.set8(state+0x134,0x88); b.set8(state+0x135,0x77); b.set8(state+0x136,0x66); b.set8(state+0x137,0x55);
        b.set8(obj+0xa4,9);
        b.q(STAGE118_CALL_35060,BtStage118Regs{r0:obj,r1:0x11,r2:0x22,r3:0x33});
    }
    fn finish_via_202e8(b:&mut B,obj:u32){
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,r1:0x81,r2:0x82,r3:0x83});
        b.q(STAGE118_CALL_21F00,BtStage118Regs{r0:obj.wrapping_add(0x100),r1:0x91,r2:0x92,r3:0x93});
        b.q(STAGE118_CALL_28A0C,BtStage118Regs{r0:0,r1:0xa1,r2:0xa2,r3:0xa3});
        b.q(STAGE118_TAIL_202E8,BtStage118Regs{r0:0xeeee,r1:0,r2:0,r3:0});
    }

    #[test]
    fn zero_35060_returns_immediately_with_exact_entry_tuple(){
        let state=0x200000;let obj=0x300000;let mut b=B::default();seed(&mut b,state,obj);
        b.ret.get_mut(&STAGE118_CALL_35060).unwrap().clear();
        b.q(STAGE118_CALL_35060,BtStage118Regs{r0:0,r1:7,r2:8,r3:9});
        assert_eq!(bt_stage118_register_state(state,1,2,3,&mut b),0);
        assert_eq!(b.calls,vec![(STAGE118_CALL_35060,BtStage118Regs{r0:0x1234_5678,r1:1,r2:2,r3:3})]);
        assert!(b.tails.is_empty());
    }

    #[test]
    fn zero_first_b082c_seeds_r8_through_19754_and_780_then_tails_221ac(){
        let state=0x210000;let obj=0x310000;let mut b=B::default();seed(&mut b,state,obj);
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:0,r1:0xa1,r2:0xa2,r3:0xa3});
        b.q(STAGE118_CALL_19754,BtStage118Regs{r0:0x55,r1:1,r2:2,r3:3});
        b.q(STAGE118_CALL_780,BtStage118Regs{r0:0x77,r1:4,r2:5,r3:6});
        b.q(STAGE118_CALL_33D28,BtStage118Regs{r0:0,r1:0x31,r2:0x32,r3:0x33});
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:0,r1:0x41,r2:0x42,r3:0x43});
        b.q(STAGE118_CALL_780,BtStage118Regs{r0:1,r1:0x51,r2:0x52,r3:0x53});
        b.q(STAGE118_CALL_19318,BtStage118Regs{r0:2,r1:0x61,r2:0x62,r3:0x63});
        b.q(STAGE118_CALL_28A0C,BtStage118Regs{r0:0,r1:0x71,r2:0x72,r3:0x73});
        b.q(STAGE118_TAIL_221AC,BtStage118Regs{r0:0xdead,r1:0,r2:0,r3:0});
        assert_eq!(bt_stage118_register_state(state,0,0,0,&mut b),0xdead);
        assert_eq!(b.nth(STAGE118_CALL_19754,0),BtStage118Regs{r0:0,r1:0xa1,r2:0xa2,r3:0xa3});
        assert_eq!(b.nth(STAGE118_CALL_780,0),BtStage118Regs{r0:1,r1:1,r2:2,r3:3});
        assert_eq!(b.nth(STAGE118_CALL_780,1),BtStage118Regs{r0:0x77,r1:0x41,r2:0x42,r3:0x43});
        assert_eq!(b.tails[0],(STAGE118_TAIL_221AC,BtStage118Regs{r0:obj,r1:0x71,r2:0x72,r3:0x73}));
    }

    #[test]
    fn byte57_is_reread_and_first_snapshot_drives_composite_and_33d28_r1_r2(){
        let state=0x220000;let obj=0x320000;let mut b=B::default();seed(&mut b,state,obj);
        b.q8(state+0x57,&[0x1c,0x00,0x04]);
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,r1:0xb1,r2:0xb2,r3:0xb3});
        b.q(STAGE118_CALL_33D28,BtStage118Regs{r0:0,r1:0xc1,r2:0xc2,r3:0xc3});
        finish_via_202e8(&mut b,obj);
        assert_eq!(bt_stage118_register_state(state,0,0,0,&mut b),0xeeee);
        assert_eq!(b.writes32.iter().find(|x|x.0==obj+0x28).unwrap().1,0x1c);
        assert_eq!(b.get8(obj+0x60),0);
        assert_eq!(b.get8(obj+0x5f),1);
        assert_eq!(b.nth(STAGE118_CALL_33D28,0),BtStage118Regs{r0:state,r1:1,r2:0x10,r3:1});
    }

    #[test]
    fn nonzero_incoming_r1_calls_3db4_and_nonzero_r2_copies_two_dwords(){
        let state=0x230000;let obj=0x330000;let pair=0x440000;let mut b=B::default();seed(&mut b,state,obj);
        b.set32(pair,0x1122_3344);b.set32(pair+4,0x5566_7788);
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,r1:0xd1,r2:0xd2,r3:0xd3});
        b.q(STAGE118_CALL_3DB4,BtStage118Regs{r0:0xe0,r1:0xe1,r2:0xe2,r3:0xe3});
        b.q(STAGE118_CALL_33D28,BtStage118Regs{r0:0,r1:0xf1,r2:0xf2,r3:0xf3});
        finish_via_202e8(&mut b,obj);
        let _=bt_stage118_register_state(state,0x99,pair,0x77,&mut b);
        assert_eq!(b.nth(STAGE118_CALL_3DB4,0),BtStage118Regs{r0:obj+0x2c,r1:0x99,r2:0x10,r3:0xd3});
        assert_eq!(b.get32(obj+0x3c),0x1122_3344);
        assert_eq!(b.get32(obj+0x40),0x5566_7788);
    }

    #[test]
    fn zero_incoming_r2_uses_fresh_state_130_134_fallback(){
        let state=0x240000;let obj=0x340000;let mut b=B::default();seed(&mut b,state,obj);
        b.set32(state+0x130,0xa1a2_a3a4);b.set32(state+0x134,0xb1b2_b3b4);
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,..Default::default()});
        b.q(STAGE118_CALL_3DB4,BtStage118Regs::default());
        b.q(STAGE118_CALL_33D28,BtStage118Regs{r0:0,..Default::default()});
        finish_via_202e8(&mut b,obj);
        let _=bt_stage118_register_state(state,1,0,0,&mut b);
        assert_eq!(b.get32(obj+0x3c),0xa1a2_a3a4);
        assert_eq!(b.get32(obj+0x40),0xb1b2_b3b4);
    }

    #[test]
    fn nonzero_33d28_r7_path_zeros_exact_fields_and_preserves_17db8_live_tuple(){
        let state=0x250000;let obj=0x350000;let mut b=B::default();seed(&mut b,state,obj);
        b.q8(state+0x57,&[0x14,0x08,0x04]);
        b.set32(obj+0x50,0);b.set32(obj+0x54,0);b.set8(obj+0x0e,0x5a);b.set8(obj+0x63,0xcc);
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,r1:0x11,r2:0x22,r3:0x33});
        b.q(STAGE118_CALL_3DB4,BtStage118Regs{r0:7,r1:8,r2:9,r3:10});
        b.q(STAGE118_CALL_33D28,BtStage118Regs{r0:1,r1:0x31,r2:0x32,r3:0x33});
        b.q(STAGE118_CALL_17DB8,BtStage118Regs{r0:0xabcd,r1:0x44,r2:0x55,r3:0x66});
        finish_via_202e8(&mut b,obj);
        let _=bt_stage118_register_state(state,1,0,0,&mut b);
        assert_eq!(b.nth(STAGE118_CALL_17DB8,0),BtStage118Regs{r0:0x5a,r1:0x31,r2:0,r3:0});
        assert_eq!(b.get32(obj+0x48),0);assert_eq!(b.get32(obj+0x4c),0);
        assert_eq!(b.get32(obj+0x50),0);assert_eq!(b.get32(obj+0x54),0);
        assert_eq!(b.get8(obj+0x65),0);assert_eq!(b.get8(obj+0x64),0);assert_eq!(b.get8(obj+0x66),0);
        assert_eq!(b.get32(obj+0x58),0xabcd);
        assert_eq!(b.get8(obj+0x5e),1);
        assert_eq!(b.get8(obj+0x63),0);
        assert_eq!(b.nth(STAGE118_CALL_B082C,1),BtStage118Regs{r0:0xabcd,r1:0x44,r2:0x55,r3:0});
    }

    #[test]
    fn nonzero_second_b082c_equal_saved_pointer_calls_202c0_then_tails_1fb24(){
        let state=0x260000;let obj=0x360000;let mut b=B::default();seed(&mut b,state,obj);
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,..Default::default()});
        b.q(STAGE118_CALL_33D28,BtStage118Regs{r0:0,..Default::default()});
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,r1:0x11,r2:0x22,r3:0x33});
        b.q(STAGE118_CALL_21F00,BtStage118Regs{r0:obj,r1:0x41,r2:0x42,r3:0x43});
        b.q(STAGE118_CALL_28A0C,BtStage118Regs{r0:0x9999,r1:0x51,r2:0x52,r3:0x53});
        b.q(STAGE118_CALL_202C0,BtStage118Regs{r0:0x61,r1:0x62,r2:0x63,r3:0x64});
        b.q(STAGE118_TAIL_1FB24,BtStage118Regs{r0:0x1111,..Default::default()});
        assert_eq!(bt_stage118_register_state(state,0,0,0,&mut b),0x1111);
        assert_eq!(b.nth(STAGE118_CALL_202C0,0),BtStage118Regs{r0:0x9999,r1:0x51,r2:0x52,r3:0x53});
        assert_eq!(b.tails[0],(STAGE118_TAIL_1FB24,BtStage118Regs{r0:0x61,r1:0x62,r2:0x63,r3:0x64}));
    }

    #[test]
    fn nonzero_second_b082c_mismatch_tails_202e8_with_28a0c_tuple(){
        let state=0x270000;let obj=0x370000;let mut b=B::default();seed(&mut b,state,obj);
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,..Default::default()});
        b.q(STAGE118_CALL_33D28,BtStage118Regs{r0:0,..Default::default()});
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,..Default::default()});
        b.q(STAGE118_CALL_21F00,BtStage118Regs{r0:0x7000,..Default::default()});
        b.q(STAGE118_CALL_28A0C,BtStage118Regs{r0:0x8000,r1:1,r2:2,r3:3});
        b.q(STAGE118_TAIL_202E8,BtStage118Regs{r0:0x2222,..Default::default()});
        assert_eq!(bt_stage118_register_state(state,0,0,0,&mut b),0x2222);
        assert_eq!(b.tails[0],(STAGE118_TAIL_202E8,BtStage118Regs{r0:0x8000,r1:1,r2:2,r3:3}));
        assert!(!b.seen(STAGE118_CALL_202C0));
    }

    #[test]
    fn zero_second_b082c_preserves_final_call_volatiles_and_selects_only_r0_for_221ac(){
        let state=0x280000;let obj=0x380000;let mut b=B::default();seed(&mut b,state,obj);
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:1,..Default::default()});
        b.q(STAGE118_CALL_33D28,BtStage118Regs{r0:0,..Default::default()});
        b.q(STAGE118_CALL_B082C,BtStage118Regs{r0:0,r1:0x11,r2:0x12,r3:0x13});
        b.q(STAGE118_CALL_780,BtStage118Regs{r0:0x21,r1:0x22,r2:0x23,r3:0x24});
        b.q(STAGE118_CALL_19318,BtStage118Regs{r0:0x31,r1:0x32,r2:0x33,r3:0x34});
        b.q(STAGE118_CALL_28A0C,BtStage118Regs{r0:0x55,r1:0x42,r2:0x43,r3:0x44});
        b.q(STAGE118_CALL_21F42,BtStage118Regs{r0:1,r1:0x52,r2:0x53,r3:0x54});
        b.q(STAGE118_CALL_21F0C,BtStage118Regs{r0:0x10,r1:0x62,r2:0x63,r3:0x64});
        b.q(STAGE118_TAIL_221AC,BtStage118Regs{r0:0x3333,..Default::default()});
        assert_eq!(bt_stage118_register_state(state,0,0,0,&mut b),0x3333);
        assert_eq!(b.nth(STAGE118_CALL_21F0C,0),BtStage118Regs{r0:0x55,r1:0x52,r2:0x53,r3:0x54});
        assert_eq!(b.tails[0],(STAGE118_TAIL_221AC,BtStage118Regs{r0:0x55,r1:0x62,r2:0x63,r3:0x64}));
    }

    #[test]
    fn provenance_and_transfer_multiset_are_exact(){
        assert_eq!(STAGE118_CURRENT_ADDR,0x16EA40);
        assert_eq!(STAGE118_LEGACY_ADDR,0x16BA74);
        assert_eq!(STAGE118_BODY_LEN,344);
        assert_eq!(STAGE118_TRANSFER_OFFSETS,[0x0C,0x18,0x1E,0x24,0x3A,0x94,0xBC,0xEE,0xF4,0xFE,0x10C,0x114,0x11C,0x122,0x126,0x12E,0x136,0x13E,0x150]);
        assert_eq!(STAGE118_DIRECT_TRANSFERS.len(),19);
        let s:BTreeSet<u32>=STAGE118_DIRECT_TRANSFERS.iter().copied().collect();
        assert_eq!(s.len(),16);
        assert!(s.contains(&STAGE118_TAIL_1FB24));
        assert!(s.contains(&STAGE118_TAIL_202E8));
        assert!(s.contains(&STAGE118_TAIL_221AC));
    }
}

pub const STAGE119_CURRENT_ADDR:u32=0x0016_EBA4;
pub const STAGE119_LEGACY_ADDR:u32=0x0016_BBD8;
pub const STAGE119_BODY_LEN:u32=64;
pub const STAGE119_CURRENT_RAW_SHA256:&str="eb6364d7f626b41ac2b70b26bb48e51022b9a2e559157f21e596e49bbc6081cd";
pub const STAGE119_NORMALIZED_SHA256:&str="a269e71c19c88a27158cb592e954183b4824ed7bd0b956ce96503cb5551e7670";
pub const STAGE119_TRANSFER_OFFSETS:[u16;3]=[0x08,0x16,0x20];
pub const STAGE119_LITERAL_POOL_START:u32=0x0016_EBE4;
pub const STAGE119_LITERAL_POOL_END:u32=0x0016_EBEC;
pub const STAGE119_NEXT_PROLOGUE:u32=0x0016_EBEC;

pub const STAGE119_CALL_338FC:u32=0x0003_38FC;
pub const STAGE119_CALL_4D552:u32=0x0004_D552;
pub const STAGE119_CALL_18540:u32=0x0001_8540;
pub const STAGE119_G_221EDC:u32=0x0022_1EDC;
pub const STAGE119_G_221EE0:u32=0x0022_1EE0;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct BtStage119Regs{pub r0:u32,pub r1:u32,pub r2:u32,pub r3:u32}

pub trait BtStage119Backend{
    fn read8(&mut self,addr:u32)->u8;
    fn read32(&mut self,addr:u32)->u32;
    fn call(&mut self,target:u32,regs:BtStage119Regs)->BtStage119Regs;
}

/// Exact current-HCD register/memory model for `0x16EBA4..0x16EBE4`.
///
/// The model preserves caller-volatile R0-R3 across all three opaque calls,
/// performs the second state+0xA4 read freshly, applies the exact 28-bit mask
/// to the dword loaded through the returned object pointer, and preserves the
/// firmware's unsigned lower/upper bound comparisons and conditional second
/// global read.
pub fn bt_stage119_register_state<B:BtStage119Backend>(
    state:u32,incoming_r1:u32,incoming_r2:u32,incoming_r3:u32,backend:&mut B
)->u32{
    let mut regs=BtStage119Regs{
        r0:u32::from(backend.read8(state.wrapping_add(0xa4))),
        r1:incoming_r1,
        r2:incoming_r2,
        r3:incoming_r3,
    };
    regs=backend.call(STAGE119_CALL_338FC,regs);
    if regs.r0==0{return 0;}

    let r4=backend.read32(regs.r0);
    if r4!=0{
        regs.r0=u32::from(backend.read8(state.wrapping_add(0xa4)));
        regs=backend.call(STAGE119_CALL_4D552,regs);
        regs.r1=backend.read32(r4.wrapping_add(0x0c))&0x0fff_ffff;
        regs=backend.call(STAGE119_CALL_18540,regs);
    }else{
        regs.r0=r4;
    }

    regs.r3=backend.read32(STAGE119_G_221EDC);
    if regs.r0<=regs.r3{return 0;}
    regs.r3=backend.read32(STAGE119_G_221EE0);
    if regs.r0<regs.r3{1}else{0}
}

#[cfg(test)]
mod stage119_tests{
    use super::*;
    use std::collections::{BTreeMap,VecDeque};
    use std::vec;
    use std::vec::Vec;

    #[derive(Default)]
    struct B{
        mem:BTreeMap<u32,u8>,
        read8q:BTreeMap<u32,VecDeque<u8>>,
        read32q:BTreeMap<u32,VecDeque<u32>>,
        ret:BTreeMap<u32,VecDeque<BtStage119Regs>>,
        calls:Vec<(u32,BtStage119Regs)>,
        reads32:Vec<u32>,
    }
    impl B{
        fn set8(&mut self,a:u32,v:u8){self.mem.insert(a,v);}
        fn set32(&mut self,a:u32,v:u32){for i in 0..4{self.set8(a.wrapping_add(i),((v>>(8*i))&0xff) as u8);}}
        fn get8(&self,a:u32)->u8{*self.mem.get(&a).unwrap_or(&0)}
        fn get32(&self,a:u32)->u32{(0..4).fold(0u32,|v,i|v|(u32::from(self.get8(a.wrapping_add(i)))<<(8*i)))}
        fn q(&mut self,t:u32,r:BtStage119Regs){self.ret.entry(t).or_default().push_back(r);}
        fn q8(&mut self,a:u32,vals:&[u8]){let q=self.read8q.entry(a).or_default();for &v in vals{q.push_back(v);}}
        fn q32(&mut self,a:u32,vals:&[u32]){let q=self.read32q.entry(a).or_default();for &v in vals{q.push_back(v);}}
    }
    impl BtStage119Backend for B{
        fn read8(&mut self,a:u32)->u8{
            if let Some(q)=self.read8q.get_mut(&a){if let Some(v)=q.pop_front(){return v;}}
            self.get8(a)
        }
        fn read32(&mut self,a:u32)->u32{
            self.reads32.push(a);
            if let Some(q)=self.read32q.get_mut(&a){if let Some(v)=q.pop_front(){return v;}}
            self.get32(a)
        }
        fn call(&mut self,t:u32,r:BtStage119Regs)->BtStage119Regs{
            self.calls.push((t,r));
            self.ret.get_mut(&t).and_then(|q|q.pop_front()).unwrap_or(r)
        }
    }

    fn base(b:&mut B,state:u32,objref:u32,obj:u32){
        b.set8(state+0xa4,3);
        b.set32(objref,obj);
        b.set32(obj+0x0c,0xf234_5678);
        b.set32(STAGE119_G_221EDC,0x1000);
        b.set32(STAGE119_G_221EE0,0x2000);
        b.q(STAGE119_CALL_338FC,BtStage119Regs{r0:objref,r1:0x11,r2:0x22,r3:0x33});
    }

    #[test]
    fn zero_338fc_returns_immediately_and_reads_no_globals(){
        let state=0x200000;let mut b=B::default();b.set8(state+0xa4,7);
        b.q(STAGE119_CALL_338FC,BtStage119Regs{r0:0,r1:9,r2:8,r3:7});
        assert_eq!(bt_stage119_register_state(state,1,2,3,&mut b),0);
        assert_eq!(b.calls,vec![(STAGE119_CALL_338FC,BtStage119Regs{r0:7,r1:1,r2:2,r3:3})]);
        assert!(b.reads32.is_empty());
    }

    #[test]
    fn zero_object_field_preserves_no_extra_calls_and_short_circuits_high_global(){
        let state=0x210000;let objref=0x300000;let mut b=B::default();
        b.set8(state+0xa4,3);b.set32(objref,0);b.set32(STAGE119_G_221EDC,0);
        b.q(STAGE119_CALL_338FC,BtStage119Regs{r0:objref,r1:0xaa,r2:0xbb,r3:0xcc});
        assert_eq!(bt_stage119_register_state(state,1,2,3,&mut b),0);
        assert_eq!(b.calls.len(),1);
        assert_eq!(b.reads32,vec![objref,STAGE119_G_221EDC]);
    }

    #[test]
    fn fresh_state_a4_and_exact_live_call_tuples_are_preserved(){
        let state=0x220000;let objref=0x310000;let obj=0x320000;let mut b=B::default();base(&mut b,state,objref,obj);
        b.q8(state+0xa4,&[3,9]);
        b.ret.get_mut(&STAGE119_CALL_338FC).unwrap().clear();
        b.q(STAGE119_CALL_338FC,BtStage119Regs{r0:objref,r1:0xa1,r2:0xa2,r3:0xa3});
        b.q(STAGE119_CALL_4D552,BtStage119Regs{r0:0x1800,r1:0xb1,r2:0xb2,r3:0xb3});
        b.q(STAGE119_CALL_18540,BtStage119Regs{r0:0x1800,r1:0xc1,r2:0xc2,r3:0xc3});
        assert_eq!(bt_stage119_register_state(state,0x11,0x22,0x33,&mut b),1);
        assert_eq!(b.calls[0],(STAGE119_CALL_338FC,BtStage119Regs{r0:3,r1:0x11,r2:0x22,r3:0x33}));
        assert_eq!(b.calls[1],(STAGE119_CALL_4D552,BtStage119Regs{r0:9,r1:0xa1,r2:0xa2,r3:0xa3}));
        assert_eq!(b.calls[2],(STAGE119_CALL_18540,BtStage119Regs{r0:0x1800,r1:0x0234_5678,r2:0xb2,r3:0xb3}));
    }

    #[test]
    fn lower_bound_equality_is_zero_and_skips_upper_read(){
        let state=0x230000;let objref=0x330000;let obj=0x340000;let mut b=B::default();base(&mut b,state,objref,obj);
        b.q(STAGE119_CALL_4D552,BtStage119Regs{r0:0x1000,..Default::default()});
        b.q(STAGE119_CALL_18540,BtStage119Regs{r0:0x1000,..Default::default()});
        assert_eq!(bt_stage119_register_state(state,0,0,0,&mut b),0);
        assert!(b.reads32.contains(&STAGE119_G_221EDC));
        assert!(!b.reads32.contains(&STAGE119_G_221EE0));
    }

    #[test]
    fn strict_inside_range_returns_one_and_reads_both_bounds(){
        let state=0x240000;let objref=0x350000;let obj=0x360000;let mut b=B::default();base(&mut b,state,objref,obj);
        b.q(STAGE119_CALL_4D552,BtStage119Regs{r0:0x1800,..Default::default()});
        b.q(STAGE119_CALL_18540,BtStage119Regs{r0:0x1800,..Default::default()});
        assert_eq!(bt_stage119_register_state(state,0,0,0,&mut b),1);
        assert!(b.reads32.contains(&STAGE119_G_221EDC));assert!(b.reads32.contains(&STAGE119_G_221EE0));
    }

    #[test]
    fn upper_bound_equality_is_zero(){
        let state=0x250000;let objref=0x370000;let obj=0x380000;let mut b=B::default();base(&mut b,state,objref,obj);
        b.q(STAGE119_CALL_4D552,BtStage119Regs{r0:0x2000,..Default::default()});
        b.q(STAGE119_CALL_18540,BtStage119Regs{r0:0x2000,..Default::default()});
        assert_eq!(bt_stage119_register_state(state,0,0,0,&mut b),0);
    }

    #[test]
    fn comparisons_are_unsigned(){
        let state=0x260000;let objref=0x390000;let obj=0x3a0000;let mut b=B::default();base(&mut b,state,objref,obj);
        b.set32(STAGE119_G_221EDC,0x8000_0000);b.set32(STAGE119_G_221EE0,0xf000_0000);
        b.q(STAGE119_CALL_4D552,BtStage119Regs{r0:0x9000_0000,..Default::default()});
        b.q(STAGE119_CALL_18540,BtStage119Regs{r0:0x9000_0000,..Default::default()});
        assert_eq!(bt_stage119_register_state(state,0,0,0,&mut b),1);
    }

    #[test]
    fn provenance_transfer_windows_and_literals_are_exact(){
        assert_eq!(STAGE119_CURRENT_ADDR,0x16EBA4);assert_eq!(STAGE119_LEGACY_ADDR,0x16BBD8);assert_eq!(STAGE119_BODY_LEN,64);
        assert_eq!(STAGE119_TRANSFER_OFFSETS,[0x08,0x16,0x20]);
        assert_eq!((STAGE119_LITERAL_POOL_START,STAGE119_LITERAL_POOL_END,STAGE119_NEXT_PROLOGUE),(0x16EBE4,0x16EBEC,0x16EBEC));
        assert_eq!((STAGE119_G_221EDC,STAGE119_G_221EE0),(0x221EDC,0x221EE0));
    }
}
