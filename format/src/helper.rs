use crate::{Soundbank, HIRCObjectBody};

pub trait SoundbankHelper {
    fn hirc_object(&self, object: &u32) -> Option<&HIRCObjectBody>;
}

impl SoundbankHelper for Soundbank {
    fn hirc_object(&self, object: &u32) -> Option<&HIRCObjectBody> {
        self.sections.iter()
            .find_map(|s| match &s.body {
                crate::SectionBody::HIRC(h) => Some(
                    h.objects.iter()
                        .find(|o| &o.id == object)
                        .map(|o| &o.body),
                ),

                _ => None,
            })
            .flatten()
    }
}
