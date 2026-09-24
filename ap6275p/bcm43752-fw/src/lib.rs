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
