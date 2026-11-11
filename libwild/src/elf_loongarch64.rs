use crate::bail;
use crate::elf::Elf64;
use crate::elf::ElfCrel;
use crate::elf::ElfRela;
use crate::elf::PLT_ENTRY_SIZE;
use crate::elf::RelocationList64;
use crate::error::Context as _;
use crate::error::Result;
use crate::platform::Platform;
use crate::platform::PreviousRelocationInfo;
use crate::platform::RelaxSymbolInfo;
use crate::platform::Relocation;
use itertools::AllEqualValueError;
use itertools::Itertools;
use linker_utils::elf::DynamicRelocationKind;
use linker_utils::elf::PAGE_MASK_4KB;
use linker_utils::elf::RelocationKind;
use linker_utils::elf::RelocationKindInfo;
use linker_utils::elf::SIZE_2KB;
use linker_utils::elf::loongarch64_rel_type_to_string;
use linker_utils::elf::shf;
use linker_utils::loongarch64::RelaxationKind;
use linker_utils::loongarch64::decode_larch_align;
use linker_utils::loongarch64::larch_align_bytes_to_remove;
use linker_utils::relaxation::RelocationModifier;
use linker_utils::relaxation::SectionRelaxDeltas;
use linker_utils::utils::or_from_slice;

pub(crate) struct ElfLoongArch64;

const PLT_ENTRY_TEMPLATE: &[u8] = &[
    0x0f, 0x0, 0x0, 0x1a, // pcalau12i $t3, offset_high(&(.got.plt[n])
    0xef, 0x1, 0xc0, 0x28, // ld.d $t3, $t3,offset_low(&(.got.plt[n])(t3)
    0xed, 0x1, 0x0, 0x4c, // jirl $t1, $t3, 0
    0x0, 0x0, 0x2a, 0x0, // break
];

const _ASSERTS: () = {
    assert!(PLT_ENTRY_TEMPLATE.len() as u64 == PLT_ENTRY_SIZE);
};

impl crate::platform::Arch for ElfLoongArch64 {
    type Relaxation = Relaxation;
    type Platform = Elf64;

    const NUM_GOT_PLT_HEADER_ENTRIES: u64 = 2;

    fn arch_identifier() -> <Self::Platform as Platform>::ArchIdentifier {
        object::elf::EM_LOONGARCH
    }

    #[inline(always)]
    fn relocation_from_raw(r_type: object::elf::RelocationType) -> Result<RelocationKindInfo> {
        linker_utils::loongarch64::relocation_type_from_raw(r_type).with_context(|| {
            format!(
                "Unsupported relocation type {}",
                Self::rel_type_to_string(r_type)
            )
        })
    }

    fn is_disallowed_for_interposable_symbols(r_type: object::elf::RelocationType) -> bool {
        matches!(r_type, object::elf::R_LARCH_32)
    }

    fn get_dynamic_relocation_type(
        relocation: DynamicRelocationKind,
    ) -> object::elf::RelocationType {
        relocation.loongarch64_r_type()
    }

    fn rel_type_to_string(r_type: object::elf::RelocationType) -> std::borrow::Cow<'static, str> {
        loongarch64_rel_type_to_string(r_type)
    }

    fn write_plt_entry(
        plt_entry: &mut [u8],
        got_address: u64,
        plt_address: u64,
    ) -> crate::error::Result {
        // TODO: For simplicity, we assume now the PLT entry precedes the GOT entry, so we can
        // make the offset calculation in the unsigned type.
        debug_assert!(plt_address < got_address);

        plt_entry.copy_from_slice(PLT_ENTRY_TEMPLATE);
        let pcala_hi20 =
            ((((got_address + SIZE_2KB) & !PAGE_MASK_4KB) - (plt_address & !PAGE_MASK_4KB)) >> 12)
                << 5;
        let pcala_lo12 = (got_address & 0xfff) << 10;
        or_from_slice(&mut plt_entry[0..4], &(pcala_hi20 as u32).to_le_bytes());
        or_from_slice(&mut plt_entry[4..8], &(pcala_lo12 as u32).to_le_bytes());
        Ok(())
    }

    fn get_dtv_offset() -> u64 {
        0
    }

    fn tp_offset_start(layout: &crate::layout::Layout<Elf64>) -> u64 {
        layout.tls_start_address_aligned()
    }

    fn get_property_class(_property_type: u32) -> Option<crate::elf::PropertyClass> {
        None
    }

    fn merge_eflags(
        mut eflags: impl Iterator<Item = object::elf::FileFlags>,
    ) -> Result<object::elf::FileFlags> {
        match eflags.all_equal_value() {
            Ok(flags) => Ok(flags),
            // no items, return blank flags
            Err(AllEqualValueError(None)) => Ok(object::elf::FileFlags(0)),
            Err(AllEqualValueError(Some([a, b]))) => bail!("non-unique e_flags: {a}, {b}"),
        }
    }

    fn high_part_relocations() -> &'static [object::elf::RelocationType] {
        &[]
    }

    fn supports_size_reduction_relaxations() -> bool {
        true
    }

    fn performs_mandatory_size_relaxations() -> bool {
        true
    }

    fn replaces_relaxation_deltas() -> bool {
        true
    }

    fn collect_relaxation_deltas(
        section_output_address: u64,
        section_bytes: &[u8],
        relocations: RelocationList64,
        _existing_deltas: Option<&SectionRelaxDeltas>,
        _resolve_symbol: impl FnMut(object::SymbolIndex) -> Option<RelaxSymbolInfo>,
    ) -> (Vec<(u64, u32)>, Option<u64>) {
        match relocations {
            RelocationList64::Rela(rela_list) => collect_relaxation_deltas(
                section_output_address,
                section_bytes,
                rela_list.iter().copied().map(ElfRela::new),
            ),
            RelocationList64::Crel(crel_iter) => collect_relaxation_deltas(
                section_output_address,
                section_bytes,
                crel_iter.flatten().map(ElfCrel::new),
            ),
        }
    }

    #[allow(unused_variables)]
    #[inline(always)]
    fn new_relaxation(
        relocation_kind: object::elf::RelocationType,
        section_bytes: &[u8],
        offset_in_section: u64,
        flags: crate::value_flags::ValueFlags,
        output_kind: crate::output_kind::OutputKind,
        section_flags: linker_utils::elf::SectionFlags,
        _relax_deltas: Option<&linker_utils::relaxation::SectionRelaxDeltas>,
        _sym_addr: u64,
        _section_address: u64,
        _rel_addend: i64,
        _previous_relocation: Option<PreviousRelocationInfo<object::elf::RelocationType>>,
    ) -> Option<Self::Relaxation>
    where
        Self: std::marker::Sized,
    {
        let mut relocation = ElfLoongArch64::relocation_from_raw(relocation_kind).unwrap();
        let interposable = flags.is_interposable();

        // All relaxations below only apply to executable code, so we shouldn't attempt them if a
        // relocation is in a non-executable section.
        if !section_flags.contains(shf::EXECINSTR) {
            return None;
        }

        let offset = offset_in_section as usize;

        match relocation_kind {
            object::elf::R_LARCH_CALL36 if !interposable => {
                relocation.kind = RelocationKind::Relative;
                return Some(Relaxation {
                    kind: RelaxationKind::NoOp,
                    rel_info: relocation,
                    mandatory: true,
                });
            }

            object::elf::R_LARCH_B26 if !interposable => {
                relocation.kind = RelocationKind::Relative;
                return Some(Relaxation {
                    kind: RelaxationKind::NoOp,
                    rel_info: relocation,
                    mandatory: true,
                });
            }

            _ => (),
        }

        None
    }

    fn get_source_info<'data>(
        object: &<Self::Platform as Platform>::File<'data>,
        relocations: &<Self::Platform as Platform>::RelocationSections,
        section: &<Self::Platform as Platform>::SectionHeader,
        offset_in_section: u64,
    ) -> Result<crate::platform::SourceInfo> {
        crate::dwarf_address_info::get_source_info::<crate::elf::Class64, Self>(
            object,
            relocations,
            section,
            offset_in_section,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Relaxation {
    kind: RelaxationKind,
    rel_info: RelocationKindInfo,
    mandatory: bool,
}

impl crate::platform::Relaxation for Relaxation {
    fn apply(&self, section_bytes: &mut [u8], offset_in_section: &mut u64, addend: &mut i64) {
        self.kind.apply(section_bytes, offset_in_section, addend);
    }

    fn rel_info(&self) -> RelocationKindInfo {
        self.rel_info
    }

    fn debug_kind(&self) -> impl std::fmt::Debug {
        &self.kind
    }

    fn next_modifier(&self) -> RelocationModifier {
        self.kind.next_modifier()
    }

    fn is_mandatory(&self) -> bool {
        self.mandatory
    }
}

/// Recompute every `R_LARCH_ALIGN` sled from the section's current address.
///
/// The returned list is the full deletion set for this pass. Deletions earlier in the section are
/// accounted for here. Counts stored by a previous pass are omitted, because the caller replaces
/// them. A margin of zero requests another pass after any section changes size: a later input
/// section moves when an earlier sled shrinks.
fn collect_relaxation_deltas<R: Relocation<Platform = Elf64>>(
    section_output_address: u64,
    section_bytes: &[u8],
    relocations: impl Iterator<Item = R>,
) -> (Vec<(u64, u32)>, Option<u64>) {
    let mut raw_deltas = Vec::new();
    let mut new_cumulative_delta = 0u64;
    let mut saw_align = false;

    for rel in relocations {
        if rel.raw_type() != object::elf::R_LARCH_ALIGN {
            continue;
        }
        saw_align = true;
        let input_offset = rel.offset();
        let addend = rel.addend();
        if addend < 0 {
            continue;
        }
        let Some(info) = decode_larch_align(rel.symbol().is_some(), addend as u64) else {
            continue;
        };
        let sled_end = input_offset.saturating_add(info.nop_bytes);
        if sled_end > section_bytes.len() as u64 {
            continue;
        }

        let output_offset = input_offset - new_cumulative_delta;
        let loc = section_output_address + output_offset;
        let remove = larch_align_bytes_to_remove(loc, info);
        if remove == 0 {
            continue;
        }
        raw_deltas.push((input_offset, remove));
        new_cumulative_delta += u64::from(remove);
    }

    let margin = if saw_align { Some(0) } else { None };
    (raw_deltas, margin)
}
