use crate::config::{Attachment, Pass, Subpass};

#[derive(PartialEq)]
pub enum AttachmentType {
    Color,
    Depth,
}

impl Attachment {}

impl Pass {
    pub fn attachments(
        &self,
    ) -> impl Iterator<Item = (&Subpass, usize, AttachmentType, &Attachment)> {
        let mut base_index = 0;
        self.subpasses.iter().flat_map(move |subpass| {
            let colors = subpass.color_attachments.iter().enumerate().map({
                let base_index = base_index;
                move |(index, color)| (subpass, base_index + index, AttachmentType::Color, color)
            });
            base_index += subpass.color_attachments.len();
            let depth = subpass.depth_attachment.iter().enumerate().map({
                let base_index = base_index;
                move |(index, depth)| (subpass, base_index + index, AttachmentType::Depth, depth)
            });
            base_index += if subpass.depth_attachment.is_some() {
                1
            } else {
                0
            };
            colors.chain(depth)
        })
    }

    pub fn writes_to_swapchain(&self) -> bool {
        self.subpasses
            .iter()
            .flat_map(|subpass| &subpass.color_attachments)
            .any(|color| color.swapchain)
    }

    pub fn subpass_index(&self, name: &str) -> usize {
        self.subpasses
            .iter()
            .enumerate()
            .find(|(_, subpass)| subpass.name == name)
            .unwrap()
            .0
    }

    pub fn swapchain_attachment_index(&self) -> usize {
        let mut attachment_index = 0;
        for subpass in &self.subpasses {
            for color in &subpass.color_attachments {
                if color.swapchain {
                    return attachment_index;
                }
                attachment_index += 1;
            }
            if subpass.depth_attachment.is_some() {
                attachment_index += 1;
            }
        }
        unreachable!()
    }

    pub fn used_as_input(&self, attachment: &Attachment) -> bool {
        for subpass in &self.subpasses {
            if subpass.input_attachment.contains(&attachment.name) {
                return true;
            }
        }
        false
    }
}

impl Subpass {
    pub fn attachment_count(&self) -> usize {
        let color_count = self.color_attachments.len();
        let depth_count = if self.depth_attachment.is_some() {
            1
        } else {
            0
        };
        color_count + depth_count
    }
}
