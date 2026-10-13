use super::*;
use crate::block::BlockType;

#[test]
fn test_inventory_starts_completely_empty() {
    let inv = Inventory::default();
    for slot in &inv.hotbar {
        assert!(slot.is_none(), "Hotbar slot must be None at start");
    }
    for slot in &inv.main {
        assert!(slot.is_none(), "Main inventory slot must be None at start");
    }
    assert_eq!(inv.selected_slot, 0);
    assert!(!inv.is_open);
    assert_eq!(inv.selected_block(), BlockType::Air);
    assert!(inv.selected_item().is_none());
}

#[test]
fn test_inventory_add_item_fills_hotbar_then_main() {
    let mut inv = Inventory::default();

    // Add 1 Wood -> should go into hotbar[0]
    let leftover = inv.add_item(BlockType::Wood, 1);
    assert_eq!(leftover, 0);
    assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockType::Wood, 1)));

    // Add 10 Dirt -> should go into hotbar[1]
    let leftover = inv.add_item(BlockType::Dirt, 10);
    assert_eq!(leftover, 0);
    assert_eq!(inv.hotbar[1], Some(ItemStack::new(BlockType::Dirt, 10)));
}

#[test]
fn test_inventory_add_item_merges_matching_stacks() {
    let mut inv = Inventory::default();

    // Fill hotbar[0] with 50 Dirt
    inv.add_item(BlockType::Dirt, 50);
    assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockType::Dirt, 50)));

    // Add 20 more Dirt -> should top off hotbar[0] to 64 and spill 6 into hotbar[1]
    let leftover = inv.add_item(BlockType::Dirt, 20);
    assert_eq!(leftover, 0);
    assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockType::Dirt, 64)));
    assert_eq!(inv.hotbar[1], Some(ItemStack::new(BlockType::Dirt, 6)));
}

#[test]
fn test_inventory_consume_selected() {
    let mut inv = Inventory::default();
    inv.hotbar[0] = Some(ItemStack::new(BlockType::Cobblestone, 2));

    assert!(inv.consume_selected(1));
    assert_eq!(
        inv.hotbar[0],
        Some(ItemStack::new(BlockType::Cobblestone, 1))
    );

    assert!(inv.consume_selected(1));
    assert_eq!(inv.hotbar[0], None);

    // Consuming from empty slot returns false
    assert!(!inv.consume_selected(1));
}

#[test]
fn test_inventory_swap_main_with_hotbar() {
    let mut inv = Inventory::default();
    inv.hotbar[0] = Some(ItemStack::new(BlockType::Wood, 5));
    inv.main[3] = Some(ItemStack::new(BlockType::DiamondOre, 12));

    inv.selected_slot = 0;
    inv.swap_main_with_active_hotbar(3);

    assert_eq!(
        inv.hotbar[0],
        Some(ItemStack::new(BlockType::DiamondOre, 12))
    );
    assert_eq!(inv.main[3], Some(ItemStack::new(BlockType::Wood, 5)));
}

#[test]
fn test_inventory_add_materials_and_tools() {
    let mut inv = Inventory::default();
    let tool = ItemType::tool(ToolType::Pickaxe, ToolTier::Wood);
    let mat = ItemType::Material(MaterialType::Stick);

    // Add tool -> only 1 goes into slot because tools don't stack
    let leftover = inv.add_item(tool, 1);
    assert_eq!(leftover, 0);
    assert_eq!(inv.hotbar[0], Some(ItemStack::new(tool, 1)));

    // Add 65 sticks -> 64 into hotbar[1], 1 into hotbar[2]
    let leftover = inv.add_item(mat, 65);
    assert_eq!(leftover, 0);
    assert_eq!(inv.hotbar[1], Some(ItemStack::new(mat, 64)));
    assert_eq!(inv.hotbar[2], Some(ItemStack::new(mat, 1)));
}
