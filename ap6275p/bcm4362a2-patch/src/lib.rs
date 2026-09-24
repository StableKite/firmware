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
    let layout=analyze_patch_layout(data).map_err(Ap6275pApplyError::Layout)?;
    if !is_ap6275p_patch_layout(&layout){return Err(Ap6275pApplyError::WrongProfile)}
    apply_hcd(data,sink).map_err(Ap6275pApplyError::Apply)
}
#[cfg(test)]mod stage14_tests{use super::*;struct S;impl PatchRamSink for S{type Error=();fn write_ram(&mut self,_:u32,_:&[u8])->Result<(),()>{Ok(())}fn launch_ram(&mut self,_:LaunchAddress)->Result<(),()>{Ok(())}}#[test]fn rejects_other_profile(){let h=[0x4c,0xfc,5,0,0,0x24,0,1,0x4e,0xfc,4,0xff,0xff,0xff,0xff];let mut s=S;assert_eq!(apply_ap6275p_hcd(&h,&mut s),Err(Ap6275pApplyError::WrongProfile));}}

/// Stage 15: validated AP6275P program object. Construction performs the exact
/// structural profile check once; applying it then cannot target another HCD image.
#[derive(Clone,Copy,Debug)]pub struct Ap6275pPatchProgram<'a>{data:&'a[u8],layout:PatchLayout}
impl<'a> Ap6275pPatchProgram<'a>{pub fn parse(data:&'a[u8])->Result<Self,PatchLayoutError>{let layout=analyze_patch_layout(data)?;if !is_ap6275p_patch_layout(&layout){return Err(PatchLayoutError::Malformed)}Ok(Self{data,layout})}pub const fn layout(&self)->PatchLayout{self.layout}pub fn apply<S:PatchRamSink>(&self,sink:&mut S)->Result<PatchApplyReport,PatchApplyError<S::Error>>{apply_hcd(self.data,sink)}}
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
pub fn validate_ap6275p_contiguous_profile(data:&[u8])->Result<Ap6275pRegionWriteCounts,PatchProfileError>{let p=Ap6275pPatchProgram::parse(data).map_err(PatchProfileError::Layout)?;let mut next_code=PATCH_CODE_START;let mut next_data=PATCH_DATA_START;let mut next_config=PATCH_CONFIG_START;let mut counts=Ap6275pRegionWriteCounts::default();p.for_each_classified_write(|r,a,d|{let next=match r{Ap6275pWriteRegion::Code=>&mut next_code,Ap6275pWriteRegion::Data=>&mut next_data,Ap6275pWriteRegion::Config=>&mut next_config};if a==*next{*next=next.wrapping_add(d.len()as u32)}else{*next=u32::MAX}match r{Ap6275pWriteRegion::Code=>{counts.code_writes+=1;counts.code_bytes+=d.len()as u32},Ap6275pWriteRegion::Data=>{counts.data_writes+=1;counts.data_bytes+=d.len()as u32},Ap6275pWriteRegion::Config=>{counts.config_writes+=1;counts.config_bytes+=d.len()as u32}}}).map_err(PatchProfileError::Layout)?;if next_code==u32::MAX{return Err(PatchProfileError::NonContiguous{region:Ap6275pWriteRegion::Code,expected:0,actual:0})}if next_data==u32::MAX{return Err(PatchProfileError::NonContiguous{region:Ap6275pWriteRegion::Data,expected:0,actual:0})}if next_config==u32::MAX{return Err(PatchProfileError::NonContiguous{region:Ap6275pWriteRegion::Config,expected:0,actual:0})}if next_code!=PATCH_CODE_END||next_data!=PATCH_DATA_END||next_config!=PATCH_CONFIG_END||counts.code_writes!=AP6275P_CODE_WRITES||counts.data_writes!=AP6275P_DATA_WRITES||counts.config_writes!=AP6275P_CONFIG_WRITES||counts.code_bytes!=AP6275P_CODE_BYTES||counts.data_bytes!=AP6275P_DATA_BYTES||counts.config_bytes!=AP6275P_CONFIG_BYTES{return Err(PatchProfileError::UnexpectedProfile)}let prof=p.layout();if prof.sentinel_launches!=1||prof.explicit_launches!=0{return Err(PatchProfileError::NonSentinelLaunch)}Ok(counts)}
#[cfg(test)]mod stage17_tests{use super::*;#[test]fn constants_match_regions(){assert_eq!(AP6275P_CODE_BYTES,PATCH_CODE_END-PATCH_CODE_START);assert_eq!(AP6275P_DATA_BYTES,PATCH_DATA_END-PATCH_DATA_START);assert_eq!(AP6275P_CONFIG_BYTES,PATCH_CONFIG_END-PATCH_CONFIG_START);assert_eq!(AP6275P_CODE_WRITES+AP6275P_DATA_WRITES+AP6275P_CONFIG_WRITES,462);assert_eq!(AP6275P_CODE_BYTES+AP6275P_DATA_BYTES+AP6275P_CONFIG_BYTES,69_895);}}
