enum Instruction {
    Cls(),
    Jump(u16),
    SetRegVX(u8, u8),
    AddValueToVX(u8, u8),
    SetIndex(u16),
    Draw(u8, u8, u8),
}
