pub trait PDASignerSeeds<const N: usize> {
    const SEED: &'static [u8];

    fn signer_seeds(&self) -> [&[u8]; N];
    fn bump_ref(&self) -> &u8;
    fn bump_as_slice(&self) -> &[u8] {
        std::slice::from_ref(self.bump_ref())
    }
}
