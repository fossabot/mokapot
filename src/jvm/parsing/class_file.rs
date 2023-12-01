use crate::{
    jvm::class::{
        BootstrapMethod, ClassFileParsingError, InnerClassInfo, NestedClassAccessFlags,
        RecordComponent,
    },
    macros::extract_attributes,
};

use super::{
    attribute::Attribute,
    parsing_context::ParsingContext,
    reader_utils::{parse_multiple, read_bytes_vec, read_u16, read_u32},
};

impl BootstrapMethod {
    fn parse<R>(reader: &mut R, ctx: &ParsingContext) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let bootstrap_method_ref = read_u16(reader)?;
        let method_ref = ctx.constant_pool.get_method_handle(bootstrap_method_ref)?;
        let num_bootstrap_arguments = read_u16(reader)?;
        let arguments = (0..num_bootstrap_arguments)
            .map(|_| {
                let arg_idx = read_u16(reader)?;
                ctx.constant_pool.get_constant_value(arg_idx)
            })
            .collect::<Result<_, _>>()?;
        Ok(BootstrapMethod {
            method: method_ref,
            arguments,
        })
    }
}

impl Attribute {
    pub fn parse_source_file<R>(
        reader: &mut R,
        ctx: &ParsingContext,
    ) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let sourcefile_index = read_u16(reader)?;
        let file_name = ctx.constant_pool.get_str(sourcefile_index)?;
        Ok(Self::SourceFile(file_name.to_owned()))
    }
    pub fn parse_innner_classes<R>(
        reader: &mut R,
        ctx: &ParsingContext,
    ) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let number_of_classes = read_u16(reader)?;
        let mut classes = Vec::with_capacity(number_of_classes as usize);
        for _ in 0..number_of_classes {
            let inner_class_info_index = read_u16(reader)?;
            let inner_class = ctx.constant_pool.get_class_ref(inner_class_info_index)?;
            let outer_class_info_index = read_u16(reader)?;
            let outer_class = if outer_class_info_index == 0 {
                None
            } else {
                let the_class = ctx.constant_pool.get_class_ref(outer_class_info_index)?;
                Some(the_class)
            };
            let inner_name_index = read_u16(reader)?;
            let inner_name = if inner_name_index == 0 {
                None
            } else {
                Some(ctx.constant_pool.get_str(inner_name_index)?.to_owned())
            };
            let access_flags = read_u16(reader)?;
            let Some(inner_class_access_flags) = NestedClassAccessFlags::from_bits(access_flags)
            else {
                return Err(ClassFileParsingError::UnknownFlags(
                    access_flags,
                    "inner class",
                ));
            };
            classes.push(InnerClassInfo {
                inner_class,
                outer_class,
                inner_name,
                inner_class_access_flags,
            });
        }
        Ok(Self::InnerClasses(classes))
    }

    pub(super) fn parse_source_debug_extension<R>(
        reader: &mut R,
        _ctx: &ParsingContext,
    ) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let attribute_length = read_u32(reader)?;
        let debug_extension = read_bytes_vec(reader, attribute_length as usize)?;
        Ok(Self::SourceDebugExtension(debug_extension))
    }

    pub(super) fn parse_bootstrap_methods<R>(
        reader: &mut R,
        ctx: &ParsingContext,
    ) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let num_bootstrap_methods = read_u16(reader)?;
        let bootstrap_methods = (0..num_bootstrap_methods)
            .map(|_| BootstrapMethod::parse(reader, ctx))
            .collect::<Result<_, _>>()?;
        Ok(Self::BootstrapMethods(bootstrap_methods))
    }
    pub(super) fn parse_nest_host<R>(
        reader: &mut R,
        ctx: &ParsingContext,
    ) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let nest_host_index = read_u16(reader)?;
        let host_class = ctx.constant_pool.get_class_ref(nest_host_index)?;
        Ok(Self::NestHost(host_class))
    }
    pub(super) fn parse_nest_members<R>(
        reader: &mut R,
        ctx: &ParsingContext,
    ) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let number_of_classes = read_u16(reader)?;
        let classes = (0..number_of_classes)
            .map(|_| {
                let class_index = read_u16(reader)?;
                ctx.constant_pool.get_class_ref(class_index)
            })
            .collect::<Result<_, _>>()?;
        Ok(Self::NestMembers(classes))
    }
    pub(super) fn parse_record<R>(
        reader: &mut R,
        ctx: &ParsingContext,
    ) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let component_count = read_u16(reader)?;
        let components = (0..component_count)
            .map(|_| {
                let name_index = read_u16(reader)?;
                let name = ctx.constant_pool.get_str(name_index)?.to_owned();
                let descriptor_index = read_u16(reader)?;
                let descriptor = ctx.constant_pool.get_str(descriptor_index)?.to_owned();

                let attributes = parse_multiple(reader, ctx, Attribute::parse)?;
                extract_attributes! {
                    for attributes in "record_component" by {
                        let signature <= Signature,
                        let rt_visible_anno <= RuntimeVisibleAnnotations,
                        let rt_invisible_anno <= RuntimeInvisibleAnnotations,
                        let rt_visible_type_anno <= RuntimeVisibleTypeAnnotations,
                        let rt_invisible_type_anno <= RuntimeInvisibleTypeAnnotations,
                    }
                }

                Ok(RecordComponent {
                    name,
                    descriptor,
                    signature,
                    runtime_visible_annotations: rt_visible_anno.unwrap_or_default(),
                    runtime_invisible_annotations: rt_invisible_anno.unwrap_or_default(),
                    runtime_visible_type_annotations: rt_visible_type_anno.unwrap_or_default(),
                    runtime_invisible_type_annotations: rt_invisible_type_anno.unwrap_or_default(),
                })
            })
            .collect::<Result<_, ClassFileParsingError>>()?;
        Ok(Self::Record(components))
    }

    pub(super) fn parse_permitted_subclasses<R>(
        reader: &mut R,
        ctx: &ParsingContext,
    ) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let number_of_classes = read_u16(reader)?;
        let classes = (0..number_of_classes)
            .map(|_| {
                let class_index = read_u16(reader)?;
                ctx.constant_pool.get_class_ref(class_index)
            })
            .collect::<Result<_, _>>()?;
        Ok(Self::PermittedSubclasses(classes))
    }
}