use cache_simulator::{
    map_strategies::{direct_map::DirectMapFactory, MapStrategyFactory},
    MemoryAddress,
};
const FACTORY: DirectMapFactory = DirectMapFactory;

fn test_mapping(
    block_size: usize,
    cache_size: usize,
    address: MemoryAddress,
    correct_map: MemoryAddress,
) {
    let mut dm = FACTORY.generate(block_size, cache_size);
    assert_eq!(dm.map(address, &[]), correct_map);
}

fn test_tag(
    block_size: usize,
    cache_size: usize,
    address: MemoryAddress,
    correct_tag: MemoryAddress,
) {
    let dm = FACTORY.generate(block_size, cache_size);
    assert_eq!(dm.get_tag(address), correct_tag);
}

#[test]
fn mapping() {
    test_mapping(4, 16, 0b1101_0010_1010, 0b10);
    test_mapping(1, 16, 0b1101_0010_1010, 0b1010);
    test_mapping(1, 1, 0b10_1010_1011_1010, 0b0);
    test_mapping(16, 1, 0b10_1010_1011_1010, 0b0);
}

#[test]
fn tags() {
    test_tag(4, 16, 0b1101_0010_1010, 0b1101);
    test_tag(1, 16, 0b1101_0010_1010, 0b110100);
    test_tag(1, 1, 0b10_1010_1011_1010, 0b101010101110);
    test_tag(16, 1, 0b10_1010_1011_1010, 0b10101010);
}
