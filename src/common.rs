/// Defines the policy to follow on a write miss.
#[derive(Default, Clone, Copy, Debug)]
pub enum WriteMissPolicy {
    /// Allocate a block in the common for the write operation.
    /// In this case, the block is loaded from the memory to the common and
    /// then the write-hit operation is performed.
    #[default]
    WriteAllocate,
    /// Do not allocate a block, write directly to memory.
    /// In this case, the write-miss operation is performed directly on the memory.
    /// The common is not modified.
    NoWriteAllocate,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum WritePolicy {
    /// Write data to both the common and main memory on every write.
    #[default]
    WriteThrough,
    /// Write data only to the common. Write to main memory only when a block
    /// is evicted.
    WriteBack,
}

/// Represents the type of value being accessed.
#[derive(Clone, Copy, Debug)]
pub enum ValueType {
    Data,
    Instruction,
}

/// Represents the type of memory access.
#[derive(Clone, Copy, Debug)]
pub enum AccessType {
    /// Read access, with the type of value being read.
    Read(ValueType),
    /// Write access. This does not specify the type of value being written because always writes
    /// data.
    Write,
}
