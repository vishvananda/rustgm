#[stable(feature = "rust1", since = "1.0.0")]
pub fn write(output: &mut dyn Write, fmt: Arguments<'_>) -> Result {
    if let Some(s) = fmt.as_str() {
        return output.write_str(s);
    }

    let mut template = fmt.template;
    let args = fmt.args;

    let mut arg_index = 0;

    // See comment on `fmt::Arguments` for the details of how the template is encoded.

    // This must match the encoding from `expand_format_args` in
    // compiler/rustc_ast_lowering/src/format.rs.
    loop {
        // SAFETY: We can assume the template is valid.
        let n = unsafe {
            let n = template.read();
            template = template.add(1);
            n
        };

        if n == 0 {
            // End of template.
            return Ok(());
        } else if n < 0x80 {
            // Literal string piece of length `n`.

            // SAFETY: We can assume the strings in the template are valid.
            let s = unsafe {
                let s = crate::str::from_raw_parts(template.as_ptr(), n as usize);
                template = template.add(n as usize);
                s
            };
            output.write_str(s)?;
        } else if n == 0x80 {
            // Literal string piece with a 16-bit length.

            // SAFETY: We can assume the strings in the template are valid.
            let s = unsafe {
                let len = usize::from(u16::from_le_bytes(template.cast_array().read()));
                template = template.add(2);
                let s = crate::str::from_raw_parts(template.as_ptr(), len);
                template = template.add(len);
                s
            };
            output.write_str(s)?;
        } else if n == 0xC0 {
            // Placeholder for next argument with default options.
            //
            // Having this as a separate case improves performance for the common case.

            // SAFETY: We can assume the template only refers to arguments that exist.
            unsafe {
                args.add(arg_index)
                    .as_ref()
                    .fmt(&mut Formatter::new(output, FormattingOptions::new()))?;
            }
            arg_index += 1;
        } else {
            // SAFETY: We can assume the template is valid.
            unsafe { assert_unchecked(n > 0xC0) };

            // Placeholder with custom options.

            let mut opt = FormattingOptions::new();

            // SAFETY: We can assume the template is valid.
            unsafe {
                if n & 1 != 0 {
                    opt.flags = u32::from_le_bytes(template.cast_array().read());
                    template = template.add(4);
                }
                if n & 2 != 0 {
                    opt.width = u16::from_le_bytes(template.cast_array().read());
                    template = template.add(2);
                }
                if n & 4 != 0 {
                    opt.precision = u16::from_le_bytes(template.cast_array().read());
                    template = template.add(2);
                }
                if n & 8 != 0 {
                    arg_index = usize::from(u16::from_le_bytes(template.cast_array().read()));
                    template = template.add(2);
                }
            }
            if n & 16 != 0 {
                // Dynamic width from a usize argument.
                // SAFETY: We can assume the template only refers to arguments that exist.
                unsafe {
                    opt.width = args.add(opt.width as usize).as_ref().as_u16().unwrap_unchecked();
                }
            }
            if n & 32 != 0 {
                // Dynamic precision from a usize argument.
                // SAFETY: We can assume the template only refers to arguments that exist.
                unsafe {
                    opt.precision =
                        args.add(opt.precision as usize).as_ref().as_u16().unwrap_unchecked();
                }
            }

            // SAFETY: We can assume the template only refers to arguments that exist.
            unsafe {
                args.add(arg_index).as_ref().fmt(&mut Formatter::new(output, opt))?;
            }
            arg_index += 1;
        }
    }
}
