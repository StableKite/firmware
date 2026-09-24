#![no_std]
//! BCM43752 FullMAC reconstruction — Stage 5.
pub mod volatile;pub const RAM_BASE:usize=0x0017_0000;
#[repr(C)]#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct OtpGeometry{pub size_field:u16,pub rows:u16,pub cols:u16}
impl OtpGeometry{pub fn set_28nm(&mut self,code:u16)->bool{match code{5=>self.rows=192,15=>self.rows=512,_=>{self.size_field=((self.rows as u32*self.cols as u32)>>4)as u16;return false;}}self.cols=32;self.size_field=((self.rows as u32*self.cols as u32)>>4)as u16;true}}
/// 32-bit firmware pointers are represented as u32 so host-side layout tests remain exact.
#[repr(C)]pub struct DnglCoreLayout32{_pad00:[u8;0x40],pub devices_addr:u32,pub ifidx_to_slot_addr:u32,_pad48:[u8;4],pub max_if:i32,pub device_capacity:i32,_pad54:[u8;8],pub ifidx_remap_addr:u32}
#[repr(C)]pub struct DnglDeviceHeader{_pad00:[u8;0x10],pub flags:u32,_pad14:[u8;3],pub ifindex:u8}
#[repr(C)]pub struct OtpContextLayout{_pad00:[u8;0x08],pub ctx_word_08:u32,_pad0c:[u8;4],pub geometry:OtpGeometry,_pad16:[u8;2],pub region_flags:u32,pub region1_start:u16,pub region1_end:u16,pub region2_start:u16,pub region2_end:u16,pub region8_start:u16,pub region8_end:u16,pub word_base:u32,_pad2c:[u8;2],pub mode_flag:u8,_pad2f:[u8;0x1d],pub extended_base:u32}
pub fn dngl_getdev_by_ifidx<'a,T>(ifidx:i32,max_if:i32,if_to_slot:&[i32],devices:&'a[T])->Option<&'a T>{if ifidx<0||ifidx>=max_if{return None}let slot=*if_to_slot.get(ifidx as usize)?;if slot<0{return None}devices.get(slot as usize)}
#[cfg(test)]extern crate std;#[cfg(test)]mod tests{use super::*;use core::mem::{offset_of,size_of};#[test]fn layouts(){assert_eq!(offset_of!(DnglCoreLayout32,devices_addr),0x40);assert_eq!(offset_of!(DnglCoreLayout32,ifidx_to_slot_addr),0x44);assert_eq!(offset_of!(DnglCoreLayout32,max_if),0x4c);assert_eq!(offset_of!(DnglCoreLayout32,device_capacity),0x50);assert_eq!(offset_of!(DnglCoreLayout32,ifidx_remap_addr),0x5c);assert_eq!(size_of::<DnglCoreLayout32>(),0x60);assert_eq!(offset_of!(DnglDeviceHeader,flags),0x10);assert_eq!(offset_of!(DnglDeviceHeader,ifindex),0x17);assert_eq!(offset_of!(OtpContextLayout,geometry),0x10);assert_eq!(offset_of!(OtpContextLayout,region_flags),0x18);assert_eq!(offset_of!(OtpContextLayout,region1_start),0x1c);assert_eq!(offset_of!(OtpContextLayout,mode_flag),0x2e);assert_eq!(offset_of!(OtpContextLayout,extended_base),0x4c);}#[test]fn otp(){let mut g=OtpGeometry::default();assert!(g.set_28nm(5));assert_eq!(g,(OtpGeometry{size_field:384,rows:192,cols:32}));}}

/// Stage 6 ROM heap ABI evidence. These are behavioral names, not recovered vendor symbols.
pub const ROM_HEAP_ALLOC_LIKE_ADDR:u32=0x0007_146C;
pub const ROM_HEAP_FREE_LIKE_ADDR:u32=0x0007_1490;
pub const ROM_HEAP_BLOCK_SIZE_LIKE_ADDR:u32=0x0007_0E10;
pub const ROM_HEAP_USAGE_OR_CONTEXT_LIKE_ADDR:u32=0x0007_1468;

/// Decouples reconstructed FullMAC code from the proprietary ROM allocator.
/// A future standalone firmware can provide a libre allocator implementation.
pub trait FirmwareHeap {
    /// # Safety
    /// Returned storage must satisfy the firmware's alignment/lifetime rules.
    unsafe fn alloc(&mut self,size:usize)->*mut u8;
    /// # Safety
    /// `ptr` must have been allocated by this heap and not already freed.
    unsafe fn free(&mut self,ptr:*mut u8);
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct RomHeapEvidence{pub address:u32,pub role:u8,pub confidence:u8}
pub const ROM_HEAP_EVIDENCE:[RomHeapEvidence;4]=[
    RomHeapEvidence{address:ROM_HEAP_ALLOC_LIKE_ADDR,role:1,confidence:3},
    RomHeapEvidence{address:ROM_HEAP_FREE_LIKE_ADDR,role:2,confidence:3},
    RomHeapEvidence{address:ROM_HEAP_BLOCK_SIZE_LIKE_ADDR,role:3,confidence:2},
    RomHeapEvidence{address:ROM_HEAP_USAGE_OR_CONTEXT_LIKE_ADDR,role:4,confidence:1},
];
#[cfg(test)]mod stage6_tests{use super::*;#[test]fn rom_heap_addresses_are_in_rom(){for x in ROM_HEAP_EVIDENCE{assert!((x.address as usize)<RAM_BASE);}}}

/// Stage 7 next ROM targets, ordered by observed call count after Stage-6 resolutions.
pub const STAGE7_NEXT_ROM_TARGETS:&[(u32,u32)]=&[
(0x6FAA4,668),
(0xF030,628),
(0xF160,428),
(0x6E274,421),
(0x6E2B4,311),
(0xEF0C,272),
(0xAE5C,266),
(0x71514,196),
(0xF2A0,187),
(0xA814,184),
(0x6E544,142),
(0x6E32C,140),
(0xA8F0,134),
(0x6E854,113),
(0x6FB24,112),
(0x1003C,106),
(0xF3F8,99),
(0xA7A8,89),
(0x6E80C,88),
(0x6E85C,81),
(0x6E878,80),
(0xB6D0,62),
(0xB298,54),
(0xBFC8,43),
(0x762E8,42),
(0x76458,41),
(0x8C8D0,41),
(0x3EB3C,40),
(0x76FE0,40),
(0x6E750,39),
(0x7FE94,37),
(0x7FDC8,35)];

/// Stage 8: high-confidence ROM stack-protector failure target.
pub const ROM_STACK_GUARD_FAIL_ADDR:u32=0x0006_FAA4;

/// Libre terminal replacement for the firmware ROM stack-canary failure sink.
/// A production port may replace the spin loop with a watchdog reset or panic path.
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
        assert!((ROM_STACK_GUARD_FAIL_ADDR as usize) < RAM_BASE);
        assert_eq!(ROM_STACK_GUARD_FAIL_ADDR, 0x6FAA4);
    }
}

/// Stage 10: two additional high-confidence BCM43752 ROM ABI resolutions.
pub const ROM_MEMCMP_ADDR:u32=0x0000_A7A8;
pub const ROM_BCM_PARSE_TLVS_ADDR:u32=0x0000_B298;

pub fn libre_memcmp(a:&[u8],b:&[u8])->i32{
    let n=core::cmp::min(a.len(),b.len());
    let mut i=0usize;
    while i<n { if a[i]!=b[i]{return a[i] as i32-b[i] as i32;} i+=1; }
    match a.len().cmp(&b.len()){core::cmp::Ordering::Less=>-1,core::cmp::Ordering::Equal=>0,core::cmp::Ordering::Greater=>1}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Tlv<'a>{pub id:u8,pub value:&'a[u8],pub offset:usize}
/// Parse Broadcom-style 1-byte id / 1-byte length / value TLVs. The returned
/// offset points at the TLV header, matching the pointer-shaped ROM result.
pub fn bcm_parse_tlvs<'a>(buf:&'a[u8],key:u8)->Option<Tlv<'a>>{
    let mut off=0usize;
    while off+2<=buf.len(){
        let id=buf[off]; let len=buf[off+1] as usize;
        let end=off.checked_add(2)?.checked_add(len)?;
        if end>buf.len(){return None;}
        if id==key{return Some(Tlv{id,value:&buf[off+2..end],offset:off});}
        off=end;
    }
    None
}
#[cfg(test)]mod stage10_tests{use super::*;#[test]fn memcmp(){assert_eq!(libre_memcmp(b"abc",b"abc"),0);assert!(libre_memcmp(b"abc",b"abd")<0);}#[test]fn tlv(){let b=[1,2,0xAA,0xBB,7,1,0xCC];let t=bcm_parse_tlvs(&b,7).unwrap();assert_eq!(t.offset,4);assert_eq!(t.value,&[0xCC]);assert!(bcm_parse_tlvs(&[1,9,0],1).is_none());}#[test]fn rom_addresses(){assert!((ROM_MEMCMP_ADDR as usize)<RAM_BASE);assert!((ROM_BCM_PARSE_TLVS_ADDR as usize)<RAM_BASE);}}

/// Stage 11/12: behaviorally resolved BCM43752 register-access ROM cluster.
/// Stage 12 corrects the provisional Stage-11 naming after checking all call-sites:
/// 0x6E544 is the stable three-argument write-like entry, while 0x6E274 and
/// 0x6E2B4 belong to the masked/modify family. Exact vendor symbol names remain unknown.
pub const ROM_REG_MODIFY_LIKE_A_ADDR:u32=0x0006_E274;
pub const ROM_REG_MODIFY_LIKE_B_ADDR:u32=0x0006_E2B4;
pub const ROM_REG_READ_LIKE_ADDR:u32=0x0006_E32C;
pub const ROM_REG_WRITE_LIKE_ADDR:u32=0x0006_E544;
#[deprecated(note="Stage-11 provisional name; 0x6E274 is modify-family, not the stable 3-argument write-like entry")]
pub const ROM_PHY_WRITE_LIKE_ADDR:u32=ROM_REG_MODIFY_LIKE_A_ADDR;
pub const ROM_PHY_MODIFY_LIKE_ADDR:u32=ROM_REG_MODIFY_LIKE_B_ADDR;
pub const ROM_PHY_READ_LIKE_ADDR:u32=ROM_REG_READ_LIKE_ADDR;
pub trait PhyRegisterIo{fn read(&mut self,reg:u16)->u16;fn write(&mut self,reg:u16,value:u16);fn modify(&mut self,reg:u16,mask:u16,value:u16){let old=self.read(reg);self.write(reg,(old&!mask)|(value&mask));}}
pub const fn phy_modify_value(old:u16,mask:u16,value:u16)->u16{(old&!mask)|(value&mask)}
#[cfg(test)]mod stage11_tests{use super::*;struct P{v:u16}impl PhyRegisterIo for P{fn read(&mut self,_:u16)->u16{self.v}fn write(&mut self,_:u16,v:u16){self.v=v}}#[test]fn modify(){assert_eq!(phy_modify_value(0xA55A,0x00F0,0x0030),0xA53A);let mut p=P{v:0xFFFF};p.modify(1,0x0F00,0x0200);assert_eq!(p.v,0xF2FF);}#[test]fn rom(){assert_eq!(ROM_REG_MODIFY_LIKE_A_ADDR,0x6E274);assert_eq!(ROM_REG_MODIFY_LIKE_B_ADDR,0x6E2B4);assert_eq!(ROM_REG_READ_LIKE_ADDR,0x6E32C);assert_eq!(ROM_REG_WRITE_LIKE_ADDR,0x6E544);}}

/// Stage 12 corrected behavioral ABI boundary. The two modify-family entries are
/// kept distinct because their exact hardware/register-bank semantics are not yet proven equal.
pub trait BroadcomRegisterAbi{fn read_like(&mut self,reg:u16)->u16;fn write_like(&mut self,reg:u16,value:u16);fn modify_like_a(&mut self,reg:u16,mask:u16,value:u16);fn modify_like_b(&mut self,reg:u16,mask:u16,value:u16);}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum RegisterOp{Write{reg:u16,value:u16},ModifyA{reg:u16,mask:u16,value:u16},ModifyB{reg:u16,mask:u16,value:u16}}
pub fn apply_register_ops<I:BroadcomRegisterAbi>(io:&mut I,ops:&[RegisterOp]){for op in ops{match *op{RegisterOp::Write{reg,value}=>io.write_like(reg,value),RegisterOp::ModifyA{reg,mask,value}=>io.modify_like_a(reg,mask,value),RegisterOp::ModifyB{reg,mask,value}=>io.modify_like_b(reg,mask,value)}}}
#[cfg(test)]mod stage12_tests{use super::*;struct R{n:u8,last:u16}impl BroadcomRegisterAbi for R{fn read_like(&mut self,_:u16)->u16{self.last}fn write_like(&mut self,_:u16,v:u16){self.n+=1;self.last=v}fn modify_like_a(&mut self,_:u16,_:u16,v:u16){self.n+=1;self.last=v}fn modify_like_b(&mut self,_:u16,_:u16,v:u16){self.n+=1;self.last=v}}#[test]fn corrected(){assert_eq!(ROM_REG_WRITE_LIKE_ADDR,0x6E544);let mut r=R{n:0,last:0};apply_register_ops(&mut r,&[RegisterOp::Write{reg:407,value:30},RegisterOp::ModifyA{reg:414,mask:0xff,value:2},RegisterOp::ModifyB{reg:87,mask:8,value:0}]);assert_eq!(r.n,3);}}

/// Stage 13: behavioral NVRAM integer-access family. Names intentionally describe
/// call shape rather than claiming exact Broadcom ROM symbol identity.
pub const ROM_NVRAM_INT_ZERO_DEFAULT_LIKE_ADDR:u32=0x0006_E80C;
pub const ROM_NVRAM_LOOKUP_RELATED_ADDR:u32=0x0006_E854;
pub const ROM_NVRAM_INT_DEFAULT_LIKE_A_ADDR:u32=0x0006_E85C;
pub const ROM_NVRAM_INT_DEFAULT_OR_INDEX_LIKE_B_ADDR:u32=0x0006_E878;
pub fn find_nvram_value<'a>(vars:&'a[u8],name:&[u8])->Option<&'a[u8]>{if name.is_empty(){return None}let mut off=0usize;while off<vars.len(){let tail=&vars[off..];let end=tail.iter().position(|&b|b==0).unwrap_or(tail.len());if end==0{return None}let e=&tail[..end];if let Some(eq)=e.iter().position(|&b|b==b'='){if &e[..eq]==name{return Some(&e[eq+1..])}}off=off.checked_add(end)?.checked_add(1)?}None}
pub fn parse_i32_auto(raw:&[u8])->Option<i32>{let mut i=0usize;while i<raw.len()&&raw[i].is_ascii_whitespace(){i+=1}let mut neg=false;if i<raw.len()&&(raw[i]==b'+'||raw[i]==b'-'){neg=raw[i]==b'-';i+=1}if i>=raw.len(){return None}let (base,start)=if i+2<=raw.len()&&raw[i]==b'0'&&(raw[i+1]==b'x'||raw[i+1]==b'X'){(16u32,i+2)}else if i+1<raw.len()&&raw[i]==b'0'{(8u32,i+1)}else{(10u32,i)};let mut p=start;let mut any=false;let mut value:u64=0;while p<raw.len(){let d=match raw[p]{b'0'..=b'9'=>(raw[p]-b'0')as u32,b'a'..=b'f'=>(raw[p]-b'a'+10)as u32,b'A'..=b'F'=>(raw[p]-b'A'+10)as u32,_=>break};if d>=base{break}value=value.checked_mul(base as u64)?.checked_add(d as u64)?;any=true;p+=1}if !any&&base==8&&start==i+1{return Some(0)}if !any{return None}if neg{if value>2147483648{return None}Some((-(value as i64))as i32)}else{if value>i32::MAX as u64{return None}Some(value as i32)}}
pub fn nvram_get_i32_default(vars:&[u8],name:&[u8],default:i32)->i32{find_nvram_value(vars,name).and_then(parse_i32_auto).unwrap_or(default)}
#[cfg(test)]mod stage13_tests{use super::*;#[test]fn nvram(){let v=b"foo=42\0hex=0x2a\0oct=052\0neg=-7\0\0";assert_eq!(nvram_get_i32_default(v,b"foo",0),42);assert_eq!(nvram_get_i32_default(v,b"hex",0),42);assert_eq!(nvram_get_i32_default(v,b"oct",0),42);assert_eq!(nvram_get_i32_default(v,b"neg",0),-7);assert_eq!(nvram_get_i32_default(v,b"missing",9),9);assert_eq!(ROM_NVRAM_INT_ZERO_DEFAULT_LIKE_ADDR,0x6E80C);}}

/// Stage 14: independent no-alloc integer-array access for Broadcom-style
/// NVRAM values. This covers the observed default/index-like ROM call shapes
/// without assigning unresolved vendor symbols to individual addresses.
pub fn nvram_int_array_value(blob:&[u8],name:&[u8],index:usize,default:i32)->i32{
    let Some(v)=find_nvram_value(blob,name) else{return default};
    let mut start=0usize;let mut idx=0usize;let mut i=0usize;
    while i<=v.len(){if i==v.len()||v[i]==b','{if idx==index{return parse_i32_auto(&v[start..i]).unwrap_or(default)}idx+=1;start=i+1}i+=1}
    default
}
pub const ROM_NVRAM_INT_ZERO_A:u32=0x0006_E80C;pub const ROM_NVRAM_INT_ZERO_B:u32=0x0006_E854;pub const ROM_NVRAM_INT_DEFAULT:u32=0x0006_E85C;pub const ROM_NVRAM_INT_ARRAY_LIKE:u32=0x0006_E878;
#[cfg(test)]mod stage14_tests{use super::*;#[test]fn array(){let b=b"a=1,0x10,-3\0b=7\0\0";assert_eq!(nvram_int_array_value(b,b"a",0,9),1);assert_eq!(nvram_int_array_value(b,b"a",1,9),16);assert_eq!(nvram_int_array_value(b,b"a",2,9),-3);assert_eq!(nvram_int_array_value(b,b"a",5,9),9);}}

/// Stage 15: a compact no-alloc view over the independently reconstructed
/// Broadcom-style NVRAM representation.
#[derive(Clone,Copy,Debug)]pub struct NvramView<'a>{blob:&'a[u8]}
impl<'a>NvramView<'a>{pub const fn new(blob:&'a[u8])->Self{Self{blob}}pub fn raw(&self,name:&[u8])->Option<&'a[u8]>{find_nvram_value(self.blob,name)}pub fn i32_or(&self,name:&[u8],default:i32)->i32{nvram_get_i32_default(self.blob,name,default)}pub fn i32_at_or(&self,name:&[u8],index:usize,default:i32)->i32{nvram_int_array_value(self.blob,name,index,default)}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct ResolvedRomBoundary{pub address:u32,pub behavior:&'static str}
pub const STAGE15_RESOLVED_ROM_BOUNDARIES:&[ResolvedRomBoundary]=&[ResolvedRomBoundary{address:ROM_MEMCMP_ADDR,behavior:"memcmp-like"},ResolvedRomBoundary{address:ROM_BCM_PARSE_TLVS_ADDR,behavior:"bcm TLV scan"},ResolvedRomBoundary{address:ROM_REG_READ_LIKE_ADDR,behavior:"PHY/register read-like"},ResolvedRomBoundary{address:ROM_REG_WRITE_LIKE_ADDR,behavior:"PHY/register write-like"},ResolvedRomBoundary{address:ROM_REG_MODIFY_LIKE_A_ADDR,behavior:"PHY/register modify-family A"},ResolvedRomBoundary{address:ROM_REG_MODIFY_LIKE_B_ADDR,behavior:"PHY/register modify-family B"},ResolvedRomBoundary{address:ROM_NVRAM_INT_ZERO_DEFAULT_LIKE_ADDR,behavior:"NVRAM integer/default family"},ResolvedRomBoundary{address:ROM_NVRAM_LOOKUP_RELATED_ADDR,behavior:"NVRAM lookup-related family"},ResolvedRomBoundary{address:ROM_NVRAM_INT_DEFAULT_LIKE_A_ADDR,behavior:"NVRAM integer/default family"},ResolvedRomBoundary{address:ROM_NVRAM_INT_DEFAULT_OR_INDEX_LIKE_B_ADDR,behavior:"NVRAM integer/index family"}];
#[cfg(test)]mod stage15_tests{use super::*;#[test]fn view(){let n=NvramView::new(b"aa=7\0arr=1,2,0x10\0\0");assert_eq!(n.i32_or(b"aa",0),7);assert_eq!(n.i32_at_or(b"arr",2,0),16);assert_eq!(n.raw(b"aa"),Some(&b"7"[..]));assert!(STAGE15_RESOLVED_ROM_BOUNDARIES.len()>=10);}}

/// Stage 16: semantic replacement surface for the recovered Broadcom NVRAM ROM
/// family, plus a conservative dependency classifier. Behavioral names remain
/// intentionally generic where the original vendor symbol is not proven.
pub enum NvramIntQuery<'a>{ZeroDefault{ name:&'a[u8]},Default{ name:&'a[u8],default:i32},Indexed{ name:&'a[u8],index:usize,default:i32}}
impl NvramView<'_>{pub fn query_i32(&self,q:NvramIntQuery<'_>)->i32{match q{NvramIntQuery::ZeroDefault{name}=>self.i32_or(name,0),NvramIntQuery::Default{name,default}=>self.i32_or(name,default),NvramIntQuery::Indexed{name,index,default}=>self.i32_at_or(name,index,default)}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum RomDependencyStatus{LibreSource,HardwareTrait,HeapBoundary,TerminalSink,Unresolved}
pub const fn rom_dependency_status(address:u32)->RomDependencyStatus{match address{
    ROM_MEMCMP_ADDR|ROM_BCM_PARSE_TLVS_ADDR|ROM_NVRAM_INT_ZERO_DEFAULT_LIKE_ADDR|ROM_NVRAM_LOOKUP_RELATED_ADDR|ROM_NVRAM_INT_DEFAULT_LIKE_A_ADDR|ROM_NVRAM_INT_DEFAULT_OR_INDEX_LIKE_B_ADDR=>RomDependencyStatus::LibreSource,
    ROM_REG_READ_LIKE_ADDR|ROM_REG_WRITE_LIKE_ADDR|ROM_REG_MODIFY_LIKE_A_ADDR|ROM_REG_MODIFY_LIKE_B_ADDR=>RomDependencyStatus::HardwareTrait,
    ROM_STACK_GUARD_FAIL_ADDR=>RomDependencyStatus::TerminalSink,
    ROM_HEAP_ALLOC_LIKE_ADDR|ROM_HEAP_FREE_LIKE_ADDR|ROM_HEAP_BLOCK_SIZE_LIKE_ADDR|ROM_HEAP_USAGE_OR_CONTEXT_LIKE_ADDR=>RomDependencyStatus::HeapBoundary,
    _=>RomDependencyStatus::Unresolved}}
#[cfg(test)]mod stage16_tests{use super::*;#[test]fn queries(){let v=NvramView::new(b"n=11\0arr=3,4,5\0\0");assert_eq!(v.query_i32(NvramIntQuery::ZeroDefault{name:b"missing"}),0);assert_eq!(v.query_i32(NvramIntQuery::Default{name:b"n",default:-1}),11);assert_eq!(v.query_i32(NvramIntQuery::Indexed{name:b"arr",index:1,default:-1}),4);}#[test]fn dependency_classes(){assert_eq!(rom_dependency_status(ROM_NVRAM_INT_ZERO_DEFAULT_LIKE_ADDR),RomDependencyStatus::LibreSource);assert_eq!(rom_dependency_status(ROM_REG_WRITE_LIKE_ADDR),RomDependencyStatus::HardwareTrait);assert_eq!(rom_dependency_status(ROM_HEAP_ALLOC_LIKE_ADDR),RomDependencyStatus::HeapBoundary);}}

/// Stage 17: weighted closure audit over the frozen Stage-7 top ROM target set.
/// Counts are observed call-site weights, not runtime frequency estimates.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct RomDependencyAudit{pub total_weight:u32,pub libre_source_weight:u32,pub hardware_trait_weight:u32,pub heap_boundary_weight:u32,pub terminal_sink_weight:u32,pub unresolved_weight:u32,pub unresolved_targets:u32}
pub fn audit_stage7_rom_dependencies()->RomDependencyAudit{let mut a=RomDependencyAudit::default();for&(addr,w)in STAGE7_NEXT_ROM_TARGETS{a.total_weight+=w;match rom_dependency_status(addr){RomDependencyStatus::LibreSource=>a.libre_source_weight+=w,RomDependencyStatus::HardwareTrait=>a.hardware_trait_weight+=w,RomDependencyStatus::HeapBoundary=>a.heap_boundary_weight+=w,RomDependencyStatus::TerminalSink=>a.terminal_sink_weight+=w,RomDependencyStatus::Unresolved=>{a.unresolved_weight+=w;a.unresolved_targets+=1}}}a}
pub fn highest_unresolved_stage7_target()->Option<(u32,u32)>{for&(addr,w)in STAGE7_NEXT_ROM_TARGETS{if rom_dependency_status(addr)==RomDependencyStatus::Unresolved{return Some((addr,w))}}None}
#[cfg(test)]mod stage17_tests{use super::*;#[test]fn weighted_closure(){let a=audit_stage7_rom_dependencies();assert_eq!(a.total_weight,5219);assert_eq!(a.libre_source_weight,505);assert_eq!(a.hardware_trait_weight,1014);assert_eq!(a.heap_boundary_weight,0);assert_eq!(a.terminal_sink_weight,668);assert_eq!(a.unresolved_weight,3032);assert_eq!(a.unresolved_targets,21);assert_eq!(highest_unresolved_stage7_target(),Some((0xF030,628)));}}


/// Stage 18: reference identity correction after centralizing the Orange Pi
/// firmware tree. Stage 6-17 addresses and closure weights above were recovered
/// from the legacy 2021 image, not from the current Orange Pi reference.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Bcm43752ReferenceIdentity {
    pub sha256: &'static str,
    pub size: u32,
    pub version: &'static str,
    pub build_date: &'static str,
    pub fwid: &'static str,
}

pub const LEGACY_RECOVERED_WIFI_REFERENCE: Bcm43752ReferenceIdentity = Bcm43752ReferenceIdentity {
    sha256: "bfcdc3ecb5274745f3c3551abd0d9b11ede89b305837241364a055fefbf09de7",
    size: 857_142,
    version: "18.35.387.23.57",
    build_date: "2021-08-03T09:39:42Z",
    fwid: "01-ea656a70",
};

pub const CURRENT_ORANGEPI_WIFI_REFERENCE: Bcm43752ReferenceIdentity = Bcm43752ReferenceIdentity {
    sha256: "6a2dbe01e72221defba91a52e158768d973a3c85ca2d881c924379e35ad36b23",
    size: 936_074,
    version: "18.35.387.23.146",
    build_date: "2022-07-12T10:55:29Z",
    fwid: "01-93c53be6",
};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Bcm43752ReconstructionReferenceStatus {
    LegacyRecovered,
    CurrentOrangePiPendingDisassembly,
}

/// The semantic recovery through Stage 17 is retained as useful legacy
/// evidence, but must not be treated as address-equivalent to the current image.
pub const STAGE18_WIFI_REFERENCE_STATUS: Bcm43752ReconstructionReferenceStatus =
    Bcm43752ReconstructionReferenceStatus::CurrentOrangePiPendingDisassembly;

#[cfg(test)]
mod stage18_tests {
    use super::*;
    #[test]
    fn references_are_distinct() {
        assert_ne!(LEGACY_RECOVERED_WIFI_REFERENCE.sha256, CURRENT_ORANGEPI_WIFI_REFERENCE.sha256);
        assert_ne!(LEGACY_RECOVERED_WIFI_REFERENCE.size, CURRENT_ORANGEPI_WIFI_REFERENCE.size);
        assert_eq!(CURRENT_ORANGEPI_WIFI_REFERENCE.version, "18.35.387.23.146");
        assert_eq!(STAGE18_WIFI_REFERENCE_STATUS, Bcm43752ReconstructionReferenceStatus::CurrentOrangePiPendingDisassembly);
    }
}

/// Stage 19: exact-byte relocation anchors from the legacy 2021 evidence image
/// into the current Orange Pi reference.  These mappings are stronger than
/// heuristic similarity: the complete legacy function/thunk byte sequence was
/// found exactly once in the current image.  They do not imply that unrelated
/// legacy addresses or the legacy call-closure remain valid.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ExactRelocationKind { FunctionBody, ThumbThunk }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ExactFunctionRelocation {
    pub legacy_address:u32,
    pub current_address:u32,
    pub byte_len:u16,
    pub name:&'static str,
    pub kind:ExactRelocationKind,
}
pub const STAGE19_WIFI_EXACT_RELOCATIONS:&[ExactFunctionRelocation]=&[
    ExactFunctionRelocation{legacy_address:0x0018_4528,current_address:0x0018_5920,byte_len:32,name:"dngl_getdev_by_ifidx",kind:ExactRelocationKind::FunctionBody},
    ExactFunctionRelocation{legacy_address:0x0018_47F4,current_address:0x0018_5BEC,byte_len:4,name:"j_dngl_sendwl",kind:ExactRelocationKind::ThumbThunk},
    ExactFunctionRelocation{legacy_address:0x001A_390E,current_address:0x001A_5CC6,byte_len:4,name:"j_hnd_free",kind:ExactRelocationKind::ThumbThunk},
    ExactFunctionRelocation{legacy_address:0x001A_45A8,current_address:0x001A_6960,byte_len:4,name:"j_nullsub_56",kind:ExactRelocationKind::ThumbThunk},
];
pub const STAGE19_WIFI_FUNCTIONS_GE8_COMPARED:u32=3_045;
pub const STAGE19_WIFI_FUNCTIONS_GE8_UNIQUE_EXACT:u32=609;
pub const STAGE19_WIFI_FUNCTIONS_GE8_MULTI_EXACT:u32=37;
pub const STAGE19_WIFI_FUNCTIONS_GE8_NO_EXACT:u32=2_399;
pub fn current_exact_address_for_legacy(legacy:u32)->Option<u32>{
    for r in STAGE19_WIFI_EXACT_RELOCATIONS{if r.legacy_address==legacy{return Some(r.current_address)}}
    None
}
#[cfg(test)]
mod stage19_tests {
    use super::*;
    #[test]fn exact_relocation_anchors(){
        assert_eq!(current_exact_address_for_legacy(0x0018_4528),Some(0x0018_5920));
        assert_eq!(STAGE19_WIFI_EXACT_RELOCATIONS[0].name,"dngl_getdev_by_ifidx");
        assert_eq!(STAGE19_WIFI_FUNCTIONS_GE8_UNIQUE_EXACT+STAGE19_WIFI_FUNCTIONS_GE8_MULTI_EXACT+STAGE19_WIFI_FUNCTIONS_GE8_NO_EXACT,STAGE19_WIFI_FUNCTIONS_GE8_COMPARED);
        assert_eq!(STAGE18_WIFI_REFERENCE_STATUS,Bcm43752ReconstructionReferenceStatus::CurrentOrangePiPendingDisassembly);
    }
}

/// Stage 20: current control-flow targets derived from exact 4-byte Thumb
/// branch thunks. The thunk instruction bytes are re-verified on the current
/// Orange Pi image before this table is committed. A target address is a
/// control-flow anchor only; it does not claim that the target body is unchanged.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ExactThunkTarget {
    pub legacy_thunk:u32,
    pub current_thunk:u32,
    pub legacy_target:u32,
    pub current_target:u32,
    pub thunk_name:&'static str,
}
pub const STAGE20_WIFI_EXACT_THUNK_TARGETS:&[ExactThunkTarget]=&[
    ExactThunkTarget{legacy_thunk:0x0018_47F4,current_thunk:0x0018_5BEC,legacy_target:0x0018_4708,current_target:0x0018_5B00,thunk_name:"j_dngl_sendwl"},
    ExactThunkTarget{legacy_thunk:0x001A_390E,current_thunk:0x001A_5CC6,legacy_target:0x001A_3C1C,current_target:0x001A_5FD4,thunk_name:"j_hnd_free"},
    ExactThunkTarget{legacy_thunk:0x001A_45A8,current_thunk:0x001A_6960,legacy_target:0x001A_4004,current_target:0x001A_63BC,thunk_name:"j_nullsub_56"},
];
pub const STAGE20_WIFI_MONOTONIC_UNIQUE_SPINE:u32=608;
pub const STAGE20_WIFI_CONTEXT_DISAMBIGUATED_EXACT:u32=29;
pub fn current_target_from_exact_thunk(legacy_thunk:u32)->Option<u32>{
    for r in STAGE20_WIFI_EXACT_THUNK_TARGETS{
        if r.legacy_thunk==legacy_thunk{return Some(r.current_target)}
    }
    None
}
#[cfg(test)]
mod stage20_tests {
    use super::*;
    #[test]
    fn exact_thunk_targets(){
        assert_eq!(current_target_from_exact_thunk(0x0018_47F4),Some(0x0018_5B00));
        assert_eq!(current_target_from_exact_thunk(0x001A_390E),Some(0x001A_5FD4));
        assert_eq!(current_target_from_exact_thunk(0x001A_45A8),Some(0x001A_63BC));
        assert_eq!(STAGE20_WIFI_MONOTONIC_UNIQUE_SPINE,608);
        assert_eq!(STAGE20_WIFI_CONTEXT_DISAMBIGUATED_EXACT,29);
    }
}

/// Stage 21: relocation-normalized current Wi-Fi closure seeded only by
/// Stage-20 current control-flow destinations.  For every entry below the
/// current function has the same complete instruction bytes as the legacy
/// function after canonicalizing only direct Thumb branch immediates. Literal
/// pool values are checked separately by the Stage-21 evidence runner.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct RelocationNormalizedFunction {
    pub legacy_address:u32,
    pub current_address:u32,
    pub byte_len:u16,
    pub name:&'static str,
}
pub const STAGE21_WIFI_RELOCATION_NORMALIZED:&[RelocationNormalizedFunction]=&[
    RelocationNormalizedFunction{legacy_address:0x0018_29E0,current_address:0x0018_2E34,byte_len:4,name:"sub_1829E0"},
    RelocationNormalizedFunction{legacy_address:0x0018_4528,current_address:0x0018_5920,byte_len:32,name:"dngl_getdev_by_ifidx"},
    RelocationNormalizedFunction{legacy_address:0x0018_4548,current_address:0x0018_5940,byte_len:50,name:"dngl_finddev"},
    RelocationNormalizedFunction{legacy_address:0x0018_4708,current_address:0x0018_5B00,byte_len:220,name:"dngl_sendwl"},
    RelocationNormalizedFunction{legacy_address:0x0018_5152,current_address:0x0018_660A,byte_len:92,name:"sub_185152"},
    RelocationNormalizedFunction{legacy_address:0x0018_5538,current_address:0x0018_69F0,byte_len:38,name:"sub_185538"},
    RelocationNormalizedFunction{legacy_address:0x001A_36D0,current_address:0x001A_5A88,byte_len:52,name:"sub_1A36D0"},
    RelocationNormalizedFunction{legacy_address:0x001A_3718,current_address:0x001A_5AD0,byte_len:56,name:"sub_1A3718"},
    RelocationNormalizedFunction{legacy_address:0x001A_39A4,current_address:0x001A_5D5C,byte_len:4,name:"sub_1A39A4"},
    RelocationNormalizedFunction{legacy_address:0x001A_39AC,current_address:0x001A_5D64,byte_len:12,name:"sub_1A39AC"},
    RelocationNormalizedFunction{legacy_address:0x001A_39BC,current_address:0x001A_5D74,byte_len:42,name:"sub_1A39BC"},
    RelocationNormalizedFunction{legacy_address:0x001A_3C1C,current_address:0x001A_5FD4,byte_len:414,name:"hnd_free"},
    RelocationNormalizedFunction{legacy_address:0x001A_45B0,current_address:0x001A_6968,byte_len:4,name:"sub_1A45B0"},
];
pub const CURRENT_DNGL_GETDEV_BY_IFIDX_ADDR:u32=0x0018_5920;
pub const CURRENT_DNGL_FINDDEV_ADDR:u32=0x0018_5940;
pub const CURRENT_DNGL_SENDWL_ADDR:u32=0x0018_5B00;
pub const CURRENT_HND_FREE_ADDR:u32=0x001A_5FD4;
pub const CURRENT_NULLSUB_56_ADDR:u32=0x001A_63BC;
pub const CURRENT_NULLSUB_56_FIRST_OPCODE:u16=0x4770; // BX LR
/// Direct-call/tail-call boundaries reached by the Stage-21 verified closure
/// that remain at the same absolute ROM address in the current image.
pub const STAGE21_WIFI_STABLE_ROM_CALL_TARGETS:&[u32]=&[
    0x0000_A814,0x0001_1D54,0x0001_2D10,0x0006_FDAC,0x0007_0718,
    0x0007_0814,0x0007_0B80,0x0007_0E10,0x0007_10CC,0x0007_10DC,
    0x0007_11C8,0x0007_1248,0x0007_616C,
];
/// Stage 6 identified 0x70e10 behaviorally; Stage 21 proves that the current
/// relocated hnd_free body still calls this same ROM boundary.
pub const CURRENT_VERIFIED_ROM_HEAP_BLOCK_SIZE_LIKE_ADDR:u32=ROM_HEAP_BLOCK_SIZE_LIKE_ADDR;
pub fn stage21_current_address_for_legacy(legacy:u32)->Option<u32>{
    for r in STAGE21_WIFI_RELOCATION_NORMALIZED{
        if r.legacy_address==legacy{return Some(r.current_address)}
    }
    None
}
#[cfg(test)]
mod stage21_tests {
    use super::*;
    #[test]
    fn normalized_destination_closure(){
        assert_eq!(STAGE21_WIFI_RELOCATION_NORMALIZED.len(),13);
        assert_eq!(stage21_current_address_for_legacy(0x0018_4548),Some(CURRENT_DNGL_FINDDEV_ADDR));
        assert_eq!(stage21_current_address_for_legacy(0x0018_4708),Some(CURRENT_DNGL_SENDWL_ADDR));
        assert_eq!(stage21_current_address_for_legacy(0x001A_3C1C),Some(CURRENT_HND_FREE_ADDR));
        assert_eq!(CURRENT_NULLSUB_56_FIRST_OPCODE,0x4770);
        assert_eq!(CURRENT_VERIFIED_ROM_HEAP_BLOCK_SIZE_LIKE_ADDR,0x0007_0E10);
        assert!(STAGE21_WIFI_STABLE_ROM_CALL_TARGETS.contains(&0x0000_A814));
        assert!(STAGE21_WIFI_STABLE_ROM_CALL_TARGETS.contains(&0x0007_0E10));
    }
}

/// Stage 22: current-image semantic rebasing for the Stage-21 verified closure.
/// ROM bodies are not present in the RAM reference; roles below are therefore
/// promoted only when current call sites, stable absolute ROM targets, and the
/// prior behavioral classification all agree.
pub const CURRENT_ROM_DIAGNOSTIC_PRINTF_LIKE_ADDR:u32=0x0000_A814;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum CurrentWifiBoundaryClass{DiagnosticSink,HeapBlockSizeLike,HndFreeInternal,Unresolved}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct CurrentWifiBoundaryAudit{pub diagnostic_calls:u8,pub heap_block_size_calls:u8,pub hnd_free_internal_calls:u8,pub unresolved_calls:u8}
pub const STAGE22_CURRENT_WIFI_BOUNDARY_AUDIT:CurrentWifiBoundaryAudit=CurrentWifiBoundaryAudit{
    diagnostic_calls:6,heap_block_size_calls:7,hnd_free_internal_calls:10,unresolved_calls:5
};
pub const fn stage22_current_wifi_boundary_class(address:u32)->CurrentWifiBoundaryClass{
    match address{
        0x0000_A814=>CurrentWifiBoundaryClass::DiagnosticSink,
        0x0007_0E10=>CurrentWifiBoundaryClass::HeapBlockSizeLike,
        0x0007_0718|0x0007_0814|0x0007_0B80|0x0007_10CC|0x0007_10DC|0x0007_11C8|0x0007_1248=>CurrentWifiBoundaryClass::HndFreeInternal,
        _=>CurrentWifiBoundaryClass::Unresolved
    }
}
/// Minimal diagnostic boundary used by the reconstructed current dngl_finddev.
pub trait DnglDiagnosticSink{fn slave_not_found(&mut self,ifidx:i32);}
/// Libre semantic replacement for current `dngl_finddev` after Stage-21
/// relocation-normalized identity and Stage-22 diagnostic-boundary verification.
/// The vendor implementation repeats the table lookup on success; returning the
/// already-resolved reference is observationally equivalent for this source model.
pub fn dngl_finddev_with_diagnostics<'a,T,D:DnglDiagnosticSink>(
    ifidx:i32,max_if:i32,if_to_slot:&[i32],devices:&'a[T],diagnostics_enabled:bool,diag:&mut D
)->Option<&'a T>{
    let found=dngl_getdev_by_ifidx(ifidx,max_if,if_to_slot,devices);
    if found.is_none()&&diagnostics_enabled{diag.slave_not_found(ifidx);}
    found
}
#[cfg(test)]
mod stage22_tests{
    use super::*;
    struct D{n:u8,last:i32}impl DnglDiagnosticSink for D{fn slave_not_found(&mut self,i:i32){self.n+=1;self.last=i}}
    #[test]fn current_boundary_audit_and_finddev(){
        assert_eq!(STAGE22_CURRENT_WIFI_BOUNDARY_AUDIT,CurrentWifiBoundaryAudit{diagnostic_calls:6,heap_block_size_calls:7,hnd_free_internal_calls:10,unresolved_calls:5});
        assert_eq!(stage22_current_wifi_boundary_class(0xA814),CurrentWifiBoundaryClass::DiagnosticSink);
        assert_eq!(stage22_current_wifi_boundary_class(0x70E10),CurrentWifiBoundaryClass::HeapBlockSizeLike);
        let slots=[1,0];let devs=[10u32,20];let mut d=D{n:0,last:0};
        assert_eq!(dngl_finddev_with_diagnostics(0,2,&slots,&devs,true,&mut d),Some(&20));assert_eq!(d.n,0);
        assert_eq!(dngl_finddev_with_diagnostics(2,2,&slots,&devs,true,&mut d),None);assert_eq!((d.n,d.last),(1,2));
    }
}

/// Stage 23: the four remaining Stage-22 Wi-Fi ROM boundaries stay
/// semantically unresolved, but their ABI call shapes are now frozen from the
/// relocation-normalized current closure. This prevents later work from
/// silently assigning an unsupported vendor symbol or argument convention.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum CurrentWifiUnresolvedCallShape {
    OneArgTail,
    TwoArgCall,
    ZeroArgCall,
    OneArgReturn,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct CurrentWifiUnresolvedBoundary {
    pub address:u32,
    pub call_count:u8,
    pub shape:CurrentWifiUnresolvedCallShape,
}
pub const STAGE23_CURRENT_WIFI_UNRESOLVED_BOUNDARIES:&[CurrentWifiUnresolvedBoundary]=&[
    CurrentWifiUnresolvedBoundary{address:0x0001_1D54,call_count:1,shape:CurrentWifiUnresolvedCallShape::OneArgTail},
    CurrentWifiUnresolvedBoundary{address:0x0001_2D10,call_count:1,shape:CurrentWifiUnresolvedCallShape::TwoArgCall},
    CurrentWifiUnresolvedBoundary{address:0x0006_FDAC,call_count:2,shape:CurrentWifiUnresolvedCallShape::ZeroArgCall},
    CurrentWifiUnresolvedBoundary{address:0x0007_616C,call_count:1,shape:CurrentWifiUnresolvedCallShape::OneArgReturn},
];
pub const fn stage23_current_wifi_unresolved_shape(address:u32)->Option<CurrentWifiUnresolvedCallShape>{
    match address{
        0x0001_1D54=>Some(CurrentWifiUnresolvedCallShape::OneArgTail),
        0x0001_2D10=>Some(CurrentWifiUnresolvedCallShape::TwoArgCall),
        0x0006_FDAC=>Some(CurrentWifiUnresolvedCallShape::ZeroArgCall),
        0x0007_616C=>Some(CurrentWifiUnresolvedCallShape::OneArgReturn),
        _=>None,
    }
}
#[cfg(test)]
mod stage23_tests{
    use super::*;
    #[test]fn unresolved_current_abi_shapes_are_frozen(){
        let mut calls=0u8;for x in STAGE23_CURRENT_WIFI_UNRESOLVED_BOUNDARIES{calls+=x.call_count}
        assert_eq!(calls,5);
        assert_eq!(stage23_current_wifi_unresolved_shape(0x11D54),Some(CurrentWifiUnresolvedCallShape::OneArgTail));
        assert_eq!(stage23_current_wifi_unresolved_shape(0x12D10),Some(CurrentWifiUnresolvedCallShape::TwoArgCall));
        assert_eq!(stage23_current_wifi_unresolved_shape(0x6FDAC),Some(CurrentWifiUnresolvedCallShape::ZeroArgCall));
        assert_eq!(stage23_current_wifi_unresolved_shape(0x7616C),Some(CurrentWifiUnresolvedCallShape::OneArgReturn));
        assert_eq!(stage23_current_wifi_unresolved_shape(0xF030),None);
    }
}

/// Stage 24: source-level lifting of two Stage-21 verified current wrappers.
/// Their unresolved ROM boundaries remain explicit traits rather than guessed
/// vendor symbols.
pub const STAGE24_CURRENT_WIFI_DIRECTIONAL_WRAPPER_ADDR:u32=0x0018_69F0;
pub const STAGE24_CURRENT_WIFI_COUNTER_CALLBACK_ADDR:u32=0x001A_5A88;
pub const STAGE24_CURRENT_WIFI_ONE_ARG_TAIL_BOUNDARY:u32=0x0001_1D54;
pub const STAGE24_CURRENT_WIFI_ONE_ARG_QUERY_BOUNDARY:u32=0x0007_616C;

pub trait CurrentWifiDirectionalCallbacks {
    fn three_arg(&mut self,context:u32,value:u32,zero:u32);
    fn one_arg(&mut self,context:u32);
}
pub trait CurrentWifiOneArgTailBoundary { fn call(&mut self,value:u32); }

/// Semantic model of current `sub_185538` / `0x1869f0`.
/// The selected optional callback runs first; the one-argument ROM boundary is
/// then always invoked with `value`.
pub fn current_wifi_directional_wrapper<C:CurrentWifiDirectionalCallbacks,T:CurrentWifiOneArgTailBoundary>(
    selector_nonzero:bool,
    three_arg_present:bool,three_arg_context:u32,
    one_arg_present:bool,one_arg_context:u32,
    value:u32,callbacks:&mut C,tail:&mut T,
){
    if selector_nonzero{
        if three_arg_present{callbacks.three_arg(three_arg_context,value,0);}
    }else if one_arg_present{callbacks.one_arg(one_arg_context);}
    tail.call(value);
}

pub trait CurrentWifiOneArgQueryBoundary { fn query(&mut self,arg:u32)->u32; }
pub trait CurrentWifiPairCallback { fn call(&mut self,context:u32,value:u32); }

/// Semantic model of current `sub_1A36D0` / `0x1A5A88`.
/// The counter is an 8-bit wrapping counter. The callback path runs only when
/// the incremented value is 0 or 1. A nonzero opaque-query result is replaced
/// with `fallback` before the pair callback.
pub fn current_wifi_wrapped_counter_callback<Q:CurrentWifiOneArgQueryBoundary,C:CurrentWifiPairCallback>(
    counter:&mut u8,callback_present:bool,query_arg:u32,fallback:u32,callback_context:u32,
    query:&mut Q,callback:&mut C,
){
    *counter=counter.wrapping_add(1);
    if *counter<=1 && callback_present{
        let mut value=query.query(query_arg);
        if value!=0{value=fallback;}
        callback.call(callback_context,value);
    }
}

#[cfg(test)]
mod stage24_tests{
    use super::*;
    #[derive(Default)]struct C{three:u8,one:u8,last:[u32;3]}
    impl CurrentWifiDirectionalCallbacks for C{
        fn three_arg(&mut self,c:u32,v:u32,z:u32){self.three+=1;self.last=[c,v,z]}
        fn one_arg(&mut self,c:u32){self.one+=1;self.last=[c,0,0]}
    }
    #[derive(Default)]struct T{n:u8,v:u32}impl CurrentWifiOneArgTailBoundary for T{fn call(&mut self,v:u32){self.n+=1;self.v=v}}
    struct Q{v:u32}impl CurrentWifiOneArgQueryBoundary for Q{fn query(&mut self,_:u32)->u32{self.v}}
    #[derive(Default)]struct P{n:u8,c:u32,v:u32}impl CurrentWifiPairCallback for P{fn call(&mut self,c:u32,v:u32){self.n+=1;self.c=c;self.v=v}}
    #[test]fn current_wrappers_preserve_order_and_wrap(){
        assert_eq!(STAGE24_CURRENT_WIFI_ONE_ARG_TAIL_BOUNDARY,0x11D54);
        assert_eq!(STAGE24_CURRENT_WIFI_ONE_ARG_QUERY_BOUNDARY,0x7616C);
        let mut c=C::default();let mut t=T::default();
        current_wifi_directional_wrapper(true,true,7,false,9,11,&mut c,&mut t);
        assert_eq!((c.three,c.one,c.last,t.n,t.v),(1,0,[7,11,0],1,11));
        current_wifi_directional_wrapper(false,false,7,true,9,12,&mut c,&mut t);
        assert_eq!((c.three,c.one,c.last,t.n,t.v),(1,1,[9,0,0],2,12));
        let mut q=Q{v:0};let mut p=P::default();let mut ctr=0u8;
        current_wifi_wrapped_counter_callback(&mut ctr,true,5,99,6,&mut q,&mut p);
        assert_eq!((ctr,p.n,p.c,p.v),(1,1,6,0));
        q.v=1;current_wifi_wrapped_counter_callback(&mut ctr,true,5,99,6,&mut q,&mut p);
        assert_eq!((ctr,p.n),(2,1));
        ctr=255;current_wifi_wrapped_counter_callback(&mut ctr,true,5,99,6,&mut q,&mut p);
        assert_eq!((ctr,p.n,p.v),(0,2,99));
    }
}
/// Stage 25: current deadman-control wrappers, verified by unique
/// relocation-normalized complete-body identity and current string/literal
/// re-reading.  The two ROM entry points remain intentionally opaque.
pub const STAGE25_CURRENT_WIFI_DEADMAN_FATAL_ADDR:u32=0x001A_5AD0;
pub const STAGE25_CURRENT_WIFI_DEADMAN_REARM_ADDR:u32=0x001A_5B14;
pub const STAGE25_CURRENT_WIFI_DEADMAN_APPLY_ADDR:u32=0x001A_5B44;
pub const STAGE25_CURRENT_WIFI_DEADMAN_STATE_MACHINE_ADDR:u32=0x001A_5B60;
pub const STAGE25_WIFI_DEADMAN_BOUNDARY_ADDR:u32=0x0001_2D10;
pub const STAGE25_WIFI_DEADMAN_ZERO_ARG_BOUNDARY_ADDR:u32=0x0006_FDAC;
pub const STAGE25_WIFI_DEADMAN_STRING_ADDR:u32=0x0020_2344;
pub const STAGE25_WIFI_DEADMAN_FORMAT_ADDR:u32=0x0020_235A;

pub trait CurrentWifiDeadmanBoundary {
    fn apply(&mut self,handle:u32,value:u32);
}

/// Semantic replacement for current `0x1A5B44`: the opaque two-argument ROM
/// boundary receives the stored value only for mode 1; every other mode passes
/// zero.  The ROM routine itself is deliberately not named.
pub fn current_wifi_deadman_apply<B:CurrentWifiDeadmanBoundary>(
    mode:u32,handle:u32,stored_value:u32,boundary:&mut B,
){
    boundary.apply(handle,if mode==1{stored_value}else{0});
}

/// Semantic replacement for current `0x1A5B14`. The firmware performs unsigned
/// wrapping subtraction and rearms only when `threshold < now-last` (strict,
/// not <=).  On rearm it stores `now` before invoking the opaque boundary.
pub fn current_wifi_deadman_rearm_if_elapsed<B:CurrentWifiDeadmanBoundary>(
    threshold:u32,now:u32,last:&mut u32,handle:u32,configured_value:u32,boundary:&mut B,
)->bool{
    if threshold!=0 && threshold<now.wrapping_sub(*last){
        *last=now;
        boundary.apply(handle,configured_value);
        true
    }else{false}
}

#[cfg(test)]
mod stage25_tests{
    use super::*;
    #[derive(Default)]struct D{n:u8,h:u32,v:u32}
    impl CurrentWifiDeadmanBoundary for D{fn apply(&mut self,h:u32,v:u32){self.n+=1;self.h=h;self.v=v}}
    #[test]fn deadman_boundary_argument_and_threshold_semantics(){
        assert_eq!(STAGE25_CURRENT_WIFI_DEADMAN_REARM_ADDR,0x1A5B14);
        assert_eq!(STAGE25_CURRENT_WIFI_DEADMAN_APPLY_ADDR,0x1A5B44);
        assert_eq!(STAGE25_WIFI_DEADMAN_BOUNDARY_ADDR,0x12D10);
        let mut d=D::default();
        current_wifi_deadman_apply(1,7,99,&mut d);assert_eq!((d.n,d.h,d.v),(1,7,99));
        current_wifi_deadman_apply(2,8,77,&mut d);assert_eq!((d.n,d.h,d.v),(2,8,0));
        let mut last=100u32;
        assert!(!current_wifi_deadman_rearm_if_elapsed(10,110,&mut last,5,6,&mut d));
        assert_eq!(last,100);
        assert!(current_wifi_deadman_rearm_if_elapsed(10,111,&mut last,5,6,&mut d));
        assert_eq!((last,d.h,d.v),(111,5,6));
        last=u32::MAX-2;
        assert!(current_wifi_deadman_rearm_if_elapsed(2,1,&mut last,9,10,&mut d));
        assert_eq!(last,1);
    }
}

/// Stage 27: complete current Wi-Fi deadman-runtime lifting. Every function
/// address below is backed by a globally unique relocation-normalized match in
/// the current Orange Pi image; unresolved ROM entries stay explicit traits.
pub const STAGE27_CURRENT_WIFI_DEADMAN_SAMPLE_CHANGE_ADDR:u32=0x001A_5C08;
pub const STAGE27_CURRENT_WIFI_DEADMAN_THRESHOLD_SAMPLE_ADDR:u32=0x001A_5C28;
pub const STAGE27_CURRENT_WIFI_OPAQUE_LOOKUP_WRAPPER_ADDR:u32=0x001A_5CA6;
pub const STAGE27_CURRENT_WIFI_SAMPLE_SOURCE_ADDR:u32=0x001A_6898;
pub const STAGE27_CURRENT_WIFI_SAMPLE_CHANGED_BOUNDARY_ADDR:u32=0x001A_67D4;
pub const STAGE27_CURRENT_WIFI_SAMPLE_EVALUATOR_ADDR:u32=0x001A_6824;
pub const STAGE27_CURRENT_WIFI_EVENT3_GATE_ADDR:u32=0x001A_6FF0;
pub const STAGE27_CURRENT_WIFI_CRITICAL_ENTER_ADDR:u32=0x001A_7310;
pub const STAGE27_CURRENT_WIFI_CRITICAL_LEAVE_ADDR:u32=0x001A_7316;
pub const STAGE27_WIFI_EVENT3_NOW_ROM_ADDR:u32=0x0006_FB24;
pub const STAGE27_WIFI_OPAQUE_LOOKUP_ROM_ADDR:u32=0x0007_03C0;
pub const STAGE27_WIFI_DEADMAN_STATE_ADDR:u32=0x0020_A504;
pub const STAGE27_WIFI_DEADMAN_OUTSTANDING_ADDR:u32=0x0020_A500;
pub const STAGE27_WIFI_DEADMAN_THRESHOLD_ADDR:u32=0x0020_A580;
pub const STAGE27_WIFI_DEADMAN_LAST_REARM_ADDR:u32=0x0020_A574;
pub const STAGE27_WIFI_DEADMAN_CONFIGURED_VALUE_ADDR:u32=0x0020_A508;
pub const STAGE27_WIFI_DEADMAN_HANDLE_ADDR:u32=0x0017_01E8;
pub const STAGE27_WIFI_SAMPLE_CHANGE_LAST_ADDR:u32=0x0020_A578;
pub const STAGE27_WIFI_THRESHOLD_SAMPLE_LAST_ADDR:u32=0x0020_A57C;
pub const STAGE27_WIFI_THRESHOLD_SAMPLE_SHIFT_ADDR:u32=0x0020_A744;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct CurrentWifiDeadmanMachineState{
    pub machine_state:u32,
    pub outstanding:u32,
    pub rearm_threshold:u32,
    pub last_rearm:u32,
    pub handle:u32,
    pub configured_value:u32,
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct CurrentWifiDeadmanObjectState{
    pub count:u32,
    pub event3_seen:u32,
}

/// Runtime-only effects that remain outside the reconstructed source boundary.
/// The method names intentionally describe caller-visible behavior rather than
/// assigning unsupported vendor symbols to the target routines.
pub trait CurrentWifiDeadmanMachineBoundary:CurrentWifiDeadmanBoundary{
    fn enter_critical(&mut self)->u32;
    fn leave_critical(&mut self,token:u32)->u32;
    fn event3_gate(&mut self)->u32;
    fn event3_now(&mut self)->u32;
    fn unexpected_event(&mut self,event:u32,state:u32);
}

/// Exact source-level transition model of current `0x1A5B60`.
/// Object offsets `+0xB0/+0xB4` are represented by `object.count/event3_seen`.
pub fn current_wifi_deadman_state_step<B:CurrentWifiDeadmanMachineBoundary>(
    state:&mut CurrentWifiDeadmanMachineState,
    object:&mut CurrentWifiDeadmanObjectState,
    event:u32,
    boundary:&mut B,
)->u32{
    let token=boundary.enter_critical();
    match state.machine_state{
        0=>{
            if event==0{
                current_wifi_deadman_apply(1,state.handle,state.configured_value,boundary);
                state.machine_state=1;
            }else{
                boundary.unexpected_event(event,0);
            }
        }
        1=>{
            if event==2{
                object.count=object.count.wrapping_add(1);
                state.outstanding=state.outstanding.wrapping_add(1);
            }else if event==3{
                object.event3_seen=1;
                object.count=object.count.wrapping_sub(1);
                state.outstanding=state.outstanding.wrapping_sub(1);
                if boundary.event3_gate()==1{
                    let now=boundary.event3_now();
                    let _=current_wifi_deadman_rearm_if_elapsed(
                        state.rearm_threshold,now,&mut state.last_rearm,
                        state.handle,state.configured_value,boundary,
                    );
                }
            }
            if state.outstanding==0 && (event&!2)==1{
                current_wifi_deadman_apply(0,state.handle,state.configured_value,boundary);
                state.machine_state=0;
            }
        }
        _=>{}
    }
    boundary.leave_critical(token)
}

pub trait CurrentWifiSampleSource{fn sample(&mut self)->u32;}
pub trait CurrentWifiSampleChangedBoundary{fn changed(&mut self)->u32;}
pub trait CurrentWifiSampleEvaluator{fn evaluate(&mut self,delta:u32,flag:u8)->u32;}

/// Current `0x1A5C08`: return zero for an unchanged sample; otherwise update the
/// stored sample and tail-dispatch the opaque change boundary.
pub fn current_wifi_sample_change<S:CurrentWifiSampleSource,B:CurrentWifiSampleChangedBoundary>(
    last:&mut u32,source:&mut S,boundary:&mut B,
)->u32{
    let now=source.sample();
    let delta=now.wrapping_sub(*last);
    if delta==0{return 0;}
    *last=now;
    boundary.changed()
}

/// ARM register-shift semantics for `MOVS r3,#1; LSLS r3,r5` in current
/// `0x1A5C28`. The shift amount is the low byte; values >=32 produce zero.
pub const fn current_wifi_lsl_one_register(shift:u32)->u32{
    let s=shift&0xff;
    if s==0{1}else if s<32{1u32<<s}else{0}
}

/// Current `0x1A5C28`: tracks a wrapping sample delta and sticky one-byte flag,
/// then tail-dispatches `(delta,flag)` to the still-opaque evaluator.
pub fn current_wifi_thresholded_sample<S:CurrentWifiSampleSource,E:CurrentWifiSampleEvaluator>(
    last:&mut u32,flag:&mut u8,shift:u32,source:&mut S,evaluator:&mut E,
)->u32{
    let now=source.sample();
    let delta=now.wrapping_sub(*last);
    if delta==0{
        *flag=0;
        return 0;
    }
    if *flag!=0 || delta>current_wifi_lsl_one_register(shift){
        *last=now;
        *flag=1;
    }
    evaluator.evaluate(delta,*flag)
}

pub trait CurrentWifiOpaqueLookupBoundary{
    fn lookup(&mut self,a0:u32,a1:u32,a2:u32,a3:u32)->u32;
}

/// Current `0x1A5CA6`: call stable opaque ROM `0x703C0` with two trailing zero
/// arguments; update outputs only on a nonzero return.
pub fn current_wifi_lookup_with_outputs<B:CurrentWifiOpaqueLookupBoundary>(
    a0:u32,a1:u32,out_a0:&mut u32,out_result:&mut u32,boundary:&mut B,
)->u32{
    let result=boundary.lookup(a0,a1,0,0);
    if result!=0{
        *out_a0=a0;
        *out_result=result;
    }
    result
}

#[cfg(test)]
mod stage27_tests{
    use super::*;
    #[derive(Default)]struct D{calls:[u8;16],n:usize,last:[u32;3],gate:u32,now:u32}
    impl CurrentWifiDeadmanBoundary for D{fn apply(&mut self,h:u32,v:u32){self.calls[self.n]=1;self.n+=1;self.last=[h,v,0]}}
    impl CurrentWifiDeadmanMachineBoundary for D{
        fn enter_critical(&mut self)->u32{self.calls[self.n]=2;self.n+=1;0x55}
        fn leave_critical(&mut self,t:u32)->u32{assert_eq!(t,0x55);self.calls[self.n]=3;self.n+=1;0xABCD}
        fn event3_gate(&mut self)->u32{self.calls[self.n]=4;self.n+=1;self.gate}
        fn event3_now(&mut self)->u32{self.calls[self.n]=5;self.n+=1;self.now}
        fn unexpected_event(&mut self,e:u32,s:u32){self.calls[self.n]=6;self.n+=1;self.last=[e,s,0]}
    }
    #[test]fn deadman_full_transition_model_and_corrected_threshold(){
        let mut s=CurrentWifiDeadmanMachineState{handle:7,configured_value:9,rearm_threshold:10,last_rearm:100,..CurrentWifiDeadmanMachineState::default()};
        let mut o=CurrentWifiDeadmanObjectState::default();let mut d=D::default();
        assert_eq!(current_wifi_deadman_state_step(&mut s,&mut o,0,&mut d),0xABCD);
        assert_eq!(s.machine_state,1);assert_eq!(&d.calls[..d.n],&[2,1,3]);assert_eq!(d.last,[7,9,0]);
        d=D::default();assert_eq!(current_wifi_deadman_state_step(&mut s,&mut o,2,&mut d),0xABCD);
        assert_eq!((o.count,s.outstanding),(1,1));assert_eq!(&d.calls[..d.n],&[2,3]);
        d=D{gate:1,now:111,..D::default()};assert_eq!(current_wifi_deadman_state_step(&mut s,&mut o,3,&mut d),0xABCD);
        assert_eq!((o.count,o.event3_seen,s.outstanding,s.machine_state,s.last_rearm),(0,1,0,0,111));
        assert_eq!(&d.calls[..d.n],&[2,4,5,1,1,3]);assert_eq!(d.last,[7,0,0]);
        let mut last=5;let mut plain=D::default();assert!(!current_wifi_deadman_rearm_if_elapsed(0,999,&mut last,1,2,&mut plain));assert_eq!(plain.n,0);
        let mut invalid=CurrentWifiDeadmanMachineState{machine_state:0,..CurrentWifiDeadmanMachineState::default()};let mut q=D::default();
        assert_eq!(current_wifi_deadman_state_step(&mut invalid,&mut o,7,&mut q),0xABCD);assert_eq!(&q.calls[..q.n],&[2,6,3]);assert_eq!(q.last,[7,0,0]);
    }
    struct S{v:u32}impl CurrentWifiSampleSource for S{fn sample(&mut self)->u32{self.v}}
    #[derive(Default)]struct C{n:u8,r:u32}impl CurrentWifiSampleChangedBoundary for C{fn changed(&mut self)->u32{self.n+=1;self.r}}
    #[derive(Default)]struct E{n:u8,d:u32,f:u8,r:u32}impl CurrentWifiSampleEvaluator for E{fn evaluate(&mut self,d:u32,f:u8)->u32{self.n+=1;self.d=d;self.f=f;self.r}}
    #[test]fn sample_helpers_preserve_wrap_shift_and_flag(){
        let mut last=10;let mut src=S{v:10};let mut c=C{r:77,..C::default()};assert_eq!(current_wifi_sample_change(&mut last,&mut src,&mut c),0);assert_eq!(c.n,0);
        src.v=12;assert_eq!(current_wifi_sample_change(&mut last,&mut src,&mut c),77);assert_eq!((last,c.n),(12,1));
        last=u32::MAX-1;src.v=1;let mut flag=0;let mut e=E{r:9,..E::default()};
        assert_eq!(current_wifi_thresholded_sample(&mut last,&mut flag,1,&mut src,&mut e),9);assert_eq!((last,flag,e.d,e.f),(1,1,3,1));
        assert_eq!(current_wifi_lsl_one_register(0),1);assert_eq!(current_wifi_lsl_one_register(5),32);assert_eq!(current_wifi_lsl_one_register(32),0);assert_eq!(current_wifi_lsl_one_register(256),1);
        src.v=1;assert_eq!(current_wifi_thresholded_sample(&mut last,&mut flag,7,&mut src,&mut e),0);assert_eq!(flag,0);
    }
    #[derive(Default)]struct L{ret:u32,args:[u32;4]}impl CurrentWifiOpaqueLookupBoundary for L{fn lookup(&mut self,a0:u32,a1:u32,a2:u32,a3:u32)->u32{self.args=[a0,a1,a2,a3];self.ret}}
    #[test]fn lookup_wrapper_preserves_zero_args_and_output_gate(){
        let mut a=90;let mut r=91;let mut l=L::default();assert_eq!(current_wifi_lookup_with_outputs(7,8,&mut a,&mut r,&mut l),0);assert_eq!((a,r,l.args),(90,91,[7,8,0,0]));
        l.ret=0x1234;assert_eq!(current_wifi_lookup_with_outputs(7,8,&mut a,&mut r,&mut l),0x1234);assert_eq!((a,r),(7,0x1234));
        assert_eq!(STAGE27_CURRENT_WIFI_DEADMAN_SAMPLE_CHANGE_ADDR,0x1A5C08);assert_eq!(STAGE27_WIFI_OPAQUE_LOOKUP_ROM_ADDR,0x703C0);
    }
}

/// Stage 28: current Wi-Fi heap/control front-end recovered from globally
/// unique relocation-normalized bodies plus current literal/string re-reading.
/// The allocator core itself remains structural-only and is not source-lifted.
pub const STAGE28_CURRENT_WIFI_HEAP_CONTEXT_GETTER_ADDR:u32=0x001A_5CCC;
pub const STAGE28_CURRENT_WIFI_HEAP_STORE_CONTEXT_ADDR:u32=0x001A_5CD8;
pub const STAGE28_CURRENT_WIFI_HEAP_SET_FLAG_200000_ADDR:u32=0x001A_5CE4;
pub const STAGE28_CURRENT_WIFI_HEAP_STATUS_REPORT_ADDR:u32=0x001A_5CF8;
pub const STAGE28_CURRENT_WIFI_HEAP_SET_FLAG_400_ADDR:u32=0x001A_5D10;
pub const STAGE28_CURRENT_WIFI_HEAP_LIST_HEAD_GETTER_ADDR:u32=0x001A_5D5C;
pub const STAGE28_CURRENT_WIFI_HEAP_RECORD_ADDRESS_ADDR:u32=0x001A_5D64;
pub const STAGE28_CURRENT_WIFI_HEAP_HANDLE_SELECT_ADDR:u32=0x001A_5D74;
pub const STAGE28_CURRENT_WIFI_HEAP_ALLOCATOR_CORE_ADDR:u32=0x001A_5DAC;
pub const STAGE28_CURRENT_WIFI_HEAP_ALLOC_FRONT_ADDR:u32=0x001A_5F1C;

pub const STAGE28_WIFI_HEAP_CONTEXT_ADDR:u32=0x0020_A584;
pub const STAGE28_WIFI_HEAP_LIST_HEAD_ADDR:u32=0x0020_9268;
pub const STAGE28_WIFI_HEAP_RECORD_BASE_ADDR:u32=0x0020_A6B0;
pub const STAGE28_WIFI_HEAP_DEFAULT_HANDLE_ADDR:u32=0x0020_A6A8;
pub const STAGE28_WIFI_HEAP_STATUS_FORMAT_ADDR:u32=0x0020_24D2;
pub const STAGE28_WIFI_HEAP_BAD_HANDLE_FORMAT_ADDR:u32=0x0020_25AC;
pub const STAGE28_WIFI_HEAP_ALLOC_SIZE_MAX:u32=0x00FF_FFFC;
pub const STAGE28_WIFI_HEAP_ALLOC_COUNTER_ADDR:u32=0x0017_0224;
pub const STAGE28_WIFI_HEAP_RECORD_STRIDE:u32=24;

#[repr(C)]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct CurrentWifiHeapContextPrefix{
    pub flags:u32,
    pub word_04:u32,
    _pad08:[u8;0x14],
    pub diagnostic_word_1c:u32,
    _pad20:[u8;0x40],
    pub word_60:u32,
}
impl Default for CurrentWifiHeapContextPrefix{
    fn default()->Self{Self{flags:0,word_04:0,_pad08:[0;0x14],diagnostic_word_1c:0,_pad20:[0;0x40],word_60:0}}
}

pub const fn current_wifi_heap_context_address()->u32{STAGE28_WIFI_HEAP_CONTEXT_ADDR}
pub fn current_wifi_heap_store_context(out:&mut u32)->u32{let p=current_wifi_heap_context_address();*out=p;p}
pub fn current_wifi_heap_set_flag_200000(ctx:&mut CurrentWifiHeapContextPrefix,value:u32)->u32{
    ctx.word_60=value;ctx.flags|=0x0020_0000;STAGE28_WIFI_HEAP_CONTEXT_ADDR
}
pub fn current_wifi_heap_set_flag_400(ctx:&mut CurrentWifiHeapContextPrefix,value:u32)->u32{
    ctx.flags|=0x400;ctx.word_04=value;STAGE28_WIFI_HEAP_CONTEXT_ADDR
}

pub trait CurrentWifiHeapStatusSink{fn report(&mut self,format_addr:u32,diagnostic_word:u32,flags:u32)->u32;}
pub fn current_wifi_heap_status_report<S:CurrentWifiHeapStatusSink>(ctx:&CurrentWifiHeapContextPrefix,sink:&mut S)->u32{
    sink.report(STAGE28_WIFI_HEAP_STATUS_FORMAT_ADDR,ctx.diagnostic_word_1c,ctx.flags)
}

pub const fn current_wifi_heap_record_address(index:u32)->u32{
    STAGE28_WIFI_HEAP_RECORD_BASE_ADDR.wrapping_add(index.wrapping_mul(STAGE28_WIFI_HEAP_RECORD_STRIDE))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum CurrentWifiHeapHandleRoute{DefaultHandle,Record(u32),FatalInvalid}
pub const fn current_wifi_heap_handle_route(handle:u32,select_record:bool)->CurrentWifiHeapHandleRoute{
    if !select_record{CurrentWifiHeapHandleRoute::DefaultHandle}
    else if handle>2{CurrentWifiHeapHandleRoute::FatalInvalid}
    else{CurrentWifiHeapHandleRoute::Record(current_wifi_heap_record_address(handle).wrapping_add(16))}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum CurrentWifiHeapAllocFrontRoute{FallbackOpaque{size:u32,class:u32},RejectTooLarge,InternalAllocator{requested:u32,aligned:u32}}
/// Front-end decision logic of current `0x1A5F1C`. For the internal route the
/// later allocator/list manipulation stays opaque; only routing and alignment
/// are reconstructed here.
pub const fn current_wifi_heap_alloc_front_route(size:u32,class:u32)->CurrentWifiHeapAllocFrontRoute{
    if current_wifi_lsl_one_register(class)>4{
        CurrentWifiHeapAllocFrontRoute::FallbackOpaque{size,class}
    }else if size>STAGE28_WIFI_HEAP_ALLOC_SIZE_MAX{
        CurrentWifiHeapAllocFrontRoute::RejectTooLarge
    }else{
        CurrentWifiHeapAllocFrontRoute::InternalAllocator{requested:size,aligned:size.wrapping_add(3)&!3}
    }
}

#[cfg(test)]
mod stage28_tests{
    use super::*;use core::mem::{offset_of,size_of};
    #[derive(Default)]struct S{args:[u32;3],ret:u32}impl CurrentWifiHeapStatusSink for S{fn report(&mut self,a:u32,b:u32,c:u32)->u32{self.args=[a,b,c];self.ret}}
    #[test]fn heap_context_layout_flags_and_status(){
        assert_eq!(offset_of!(CurrentWifiHeapContextPrefix,flags),0);assert_eq!(offset_of!(CurrentWifiHeapContextPrefix,word_04),4);
        assert_eq!(offset_of!(CurrentWifiHeapContextPrefix,diagnostic_word_1c),0x1c);assert_eq!(offset_of!(CurrentWifiHeapContextPrefix,word_60),0x60);assert_eq!(size_of::<CurrentWifiHeapContextPrefix>(),0x64);
        let mut c=CurrentWifiHeapContextPrefix::default();let mut out=0;assert_eq!(current_wifi_heap_store_context(&mut out),STAGE28_WIFI_HEAP_CONTEXT_ADDR);assert_eq!(out,STAGE28_WIFI_HEAP_CONTEXT_ADDR);
        assert_eq!(current_wifi_heap_set_flag_200000(&mut c,0x55),STAGE28_WIFI_HEAP_CONTEXT_ADDR);assert_eq!((c.flags,c.word_60),(0x20_0000,0x55));
        assert_eq!(current_wifi_heap_set_flag_400(&mut c,0x66),STAGE28_WIFI_HEAP_CONTEXT_ADDR);assert_eq!((c.flags,c.word_04),(0x20_0400,0x66));
        c.diagnostic_word_1c=0x77;let mut s=S{ret:9,..S::default()};assert_eq!(current_wifi_heap_status_report(&c,&mut s),9);assert_eq!(s.args,[STAGE28_WIFI_HEAP_STATUS_FORMAT_ADDR,0x77,0x20_0400]);
    }
    #[test]fn handle_and_alloc_front_routes_preserve_weird_edges(){
        assert_eq!(current_wifi_heap_record_address(2),STAGE28_WIFI_HEAP_RECORD_BASE_ADDR+48);
        assert_eq!(current_wifi_heap_handle_route(99,false),CurrentWifiHeapHandleRoute::DefaultHandle);
        assert_eq!(current_wifi_heap_handle_route(2,true),CurrentWifiHeapHandleRoute::Record(STAGE28_WIFI_HEAP_RECORD_BASE_ADDR+64));
        assert_eq!(current_wifi_heap_handle_route(3,true),CurrentWifiHeapHandleRoute::FatalInvalid);
        assert_eq!(current_wifi_heap_alloc_front_route(7,0),CurrentWifiHeapAllocFrontRoute::InternalAllocator{requested:7,aligned:8});
        assert_eq!(current_wifi_heap_alloc_front_route(STAGE28_WIFI_HEAP_ALLOC_SIZE_MAX+1,0),CurrentWifiHeapAllocFrontRoute::RejectTooLarge);
        assert_eq!(current_wifi_heap_alloc_front_route(7,3),CurrentWifiHeapAllocFrontRoute::FallbackOpaque{size:7,class:3});
        // ARM register shift uses only low byte: 256 behaves like shift zero.
        assert_eq!(current_wifi_heap_alloc_front_route(7,256),CurrentWifiHeapAllocFrontRoute::InternalAllocator{requested:7,aligned:8});
        assert_eq!(STAGE28_CURRENT_WIFI_HEAP_ALLOCATOR_CORE_ADDR,0x1A5DAC);
    }
}
