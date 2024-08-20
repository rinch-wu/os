#[allow(clippy::upper_case_acronyms)]
pub struct PLIC {
    base_addr: usize,
}

pub enum IntrTargetPriority {
    Machine = 0,
    Supervisor = 1,
}

impl IntrTargetPriority {
    pub fn supported_number() -> usize {
        2
    }
}

impl PLIC {
    pub unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    fn hart_id_with_priority(hart_id: usize, target_priority: IntrTargetPriority) -> usize {
        let priority_num = IntrTargetPriority::supported_number();
        hart_id * priority_num + target_priority as usize
    }

    fn claim_comp_ptr_of_hart_with_priority(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
    ) -> *mut u32 {
        let id = Self::hart_id_with_priority(hart_id, target_priority);
        (self.base_addr + 0x20_0004 + 0x1000 * id) as *mut u32
    }

    pub fn cliam(&mut self, hart_id: usize, target_priority: IntrTargetPriority) -> u32 {
        let claim_comp_ptr = self.claim_comp_ptr_of_hart_with_priority(hart_id, target_priority);
        unsafe { claim_comp_ptr.read_volatile() }
    }
    pub fn complete(
        &mut self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        completion: u32,
    ) {
        let claim_comp_ptr = self.claim_comp_ptr_of_hart_with_priority(hart_id, target_priority);
        unsafe {
            claim_comp_ptr.write_volatile(completion);
        }
    }
}
