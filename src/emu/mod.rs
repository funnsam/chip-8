macro_rules! mod_use {
    ($($file: ident), *) => {
        $(
            pub mod $file;
            pub use $file::*;
        )*
    };
}

// mod_use!(cpu);
// mod_use!(screen);
// mod_use!(config);
// mod_use!(font);

mod_use!(
    cpu, screen, config, font
);
