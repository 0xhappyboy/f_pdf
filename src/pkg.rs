//! Package-level structures for the PDF document.
#[derive(Debug, Clone, Default)]
pub struct FPackagePart {
    /// Object number in the PDF cross-reference table.
    pub object_number: u32,
    /// Generation number of the object.
    pub generation: u32,
    /// Raw bytes of the object content.
    pub data: Vec<u8>,
}
/// The entry point relationships and every object that was not parsed into a dedicated structure.
#[derive(Debug, Clone, Default)]
pub struct FPackage {
    pub main_part: String,
    pub parts: Vec<FPackagePart>,
}
impl FPackage {
    /// Returns the bytes of a part by object number, if present.
    pub fn get_part(&self, object_number: u32) -> Option<&[u8]> {
        self.parts
            .iter()
            .find(|p| p.object_number == object_number)
            .map(|p| p.data.as_slice())
    }
    /// Inserts or replaces a part.
    pub fn set_part(&mut self, object_number: u32, generation: u32, data: Vec<u8>) {
        if let Some(existing) = self
            .parts
            .iter_mut()
            .find(|p| p.object_number == object_number)
        {
            existing.generation = generation;
            existing.data = data;
        } else {
            self.parts.push(FPackagePart {
                object_number,
                generation,
                data,
            });
        }
    }
    /// Removes a part by object number.
    pub fn remove_part(&mut self, object_number: u32) {
        self.parts.retain(|p| p.object_number != object_number);
    }
}
