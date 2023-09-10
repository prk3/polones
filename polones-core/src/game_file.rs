pub struct GameFile {
    pub name: String,
    data: Vec<u8>,
    pub format: FileFormat,
    pub mapper: u16,
    pub submapper: Option<u8>,
    trainer: Option<(usize, usize)>,
    prg_rom: (usize, usize),
    chr_rom: Option<(usize, usize)>,
    pub mirroring_vertical: bool,
    pub battery_present: bool,
    pub four_screen_mode: bool,

    pub prg_ram_size: Option<usize>,
    pub prg_nvram_size: Option<usize>,
    pub chr_ram_size: Option<usize>,
    pub chr_nvram_size: Option<usize>,
}

impl std::fmt::Debug for GameFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("GameFile");
        s.field("name", &self.name);
        s.field(
            "data",
            &("length", self.data.len(), "header", &self.data[0..16]),
        );
        s.field("format", &self.format);
        s.field("mapper", &self.mapper);
        s.field("submapper", &self.submapper);
        s.field("trainer", &self.trainer);
        s.field("prg_rom", &self.prg_rom);
        s.field("chr_rom", &self.chr_rom);
        s.field("mirroring_vertical", &self.mirroring_vertical);
        s.field("battery_present", &self.battery_present);
        s.field("four_screen_mode", &self.four_screen_mode);
        s.field("prg_ram_size", &self.prg_ram_size);
        s.field("prg_nvram_size", &self.prg_nvram_size);
        s.field("chr_ram_size", &self.chr_ram_size);
        s.field("chr_nvram_size", &self.chr_nvram_size);
        s.finish()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileFormat {
    ArchaicINes,
    INes,
    Nes20,
}

impl GameFile {
    pub fn read(name: String, data: Vec<u8>) -> Result<Self, ()> {
        let mut read: usize = 0;
        let assert_has_bytes = |bytes: usize| {
            if data.len() >= bytes {
                Ok(())
            } else {
                Err(())
            }
        };

        // Make sure data contains header.
        assert_has_bytes(read + 16)?;
        read += 16;

        // Make sure it's a .nes file.
        if &data[0..=3] != b"NES\x1A" {
            return Err(());
        }

        // Read data common to all file formats.
        let prg_rom_size_lsb = data[4];
        let chr_rom_size_lsb = data[5];
        let mapper_number_nybble_1 = data[6] >> 4;
        let four_screen_mode = data[6] & 0b1000 > 0;
        let trainer_present = data[6] & 0b0100 > 0;
        let battery_present = data[6] & 0b0010 > 0;
        let mirroring_vertical = data[6] & 0b0001 > 0;

        let trainer = if trainer_present {
            let start = read;
            assert_has_bytes(read + 512)?;
            read += 512;
            Some((start, read))
        } else {
            None
        };

        // Read rom sizes as if it was NES 2.0 format. We'll later check if it
        // conforms to that format.
        let chr_rom_size_msb = data[9] >> 4;
        let prg_rom_size_msb = data[9] & 0b00001111;

        // Calculate NES 2.0 rom sizes.
        let prg_rom_size = if prg_rom_size_msb < 0xf {
            ((prg_rom_size_msb as usize) << 8 | prg_rom_size_lsb as usize) * 16384
        } else {
            let multiplier = (prg_rom_size_lsb & 0b00000011) * 2 + 1;
            let exponent = prg_rom_size_lsb >> 2;
            2usize.pow(exponent as u32) * multiplier as usize
        };

        let chr_rom_size = if chr_rom_size_msb < 0xf {
            ((chr_rom_size_msb as usize) << 8 | chr_rom_size_lsb as usize) * 8192
        } else {
            let multiplier = (chr_rom_size_lsb & 0b00000011) * 2 + 1;
            let exponent = chr_rom_size_lsb >> 2;
            2usize.pow(exponent as u32) * multiplier as usize
        };

        let format: FileFormat;
        let mapper: u16;
        let prg_rom: (usize, usize);
        let chr_rom: Option<(usize, usize)>;
        let mut prg_ram_size: Option<usize> = None;
        let mut prg_nvram_size: Option<usize> = None;
        let mut chr_ram_size: Option<usize> = None;
        let mut chr_nvram_size: Option<usize> = None;
        let mut submapper: Option<u8> = None;

        // Now we have enough data to decide which format we're dealing with.
        // If it's not NES 2.0, we'll reinterpret byte 9.
        if data[7] & 0b1100 == 0b1000
            && data.len() >= (16 + (trainer_present as usize * 512) + prg_rom_size + chr_rom_size)
        {
            // NES 2.0
            format = FileFormat::Nes20;

            let mapper_number_nybble_2 = data[7] >> 4;
            let console_type = data[7] & 0b00000011;

            submapper = Some(data[8] >> 4);

            let mapper_number_nybble_3 = data[8] & 0b00001111;

            let prg_nvram_size_shift = data[10] >> 4;
            let prg_ram_size_shift = data[10] & 0b00001111;

            let chr_nvram_size_shift = data[11] >> 4;
            let chr_ram_size_shift = data[11] & 0b00001111;

            let _cpu_ppu_timing_mode = data[12] & 0b00000011;

            if chr_nvram_size_shift > 0 && !battery_present {
                return Err(());
            }

            let _hardware_type = if console_type == 1 {
                Some(data[13] >> 4)
            } else {
                None
            };
            let _ppu_type = if console_type == 1 {
                Some(data[13] & 0b00001111)
            } else {
                None
            };
            let _extended_console_type = if console_type == 3 {
                Some(data[13] & 0b00001111)
            } else {
                None
            };

            let _miscellaneous_roms_number = data[14] & 0b00000011;
            let _default_expansion_device = data[15] & 0b00111111;

            mapper = (mapper_number_nybble_3 as u16) << 8
                | (mapper_number_nybble_2 as u16) << 4
                | mapper_number_nybble_1 as u16;

            prg_rom = {
                let start = read;
                assert_has_bytes(read + prg_rom_size)?;
                read += prg_rom_size;
                (start, read)
            };

            chr_rom = if chr_rom_size > 0 {
                let start = read;
                assert_has_bytes(read + chr_rom_size)?;
                read += chr_rom_size;
                Some((start, read))
            } else {
                None
            };

            prg_nvram_size = if prg_nvram_size_shift > 0 {
                Some(64 << prg_nvram_size_shift)
            } else {
                None
            };
            prg_ram_size = if prg_ram_size_shift > 0 {
                Some(64 << prg_ram_size_shift)
            } else {
                None
            };
            chr_nvram_size = if chr_nvram_size_shift > 0 {
                Some(64 << chr_nvram_size_shift)
            } else {
                None
            };
            chr_ram_size = if chr_ram_size_shift > 0 {
                Some(64 << chr_ram_size_shift)
            } else {
                None
            };
        } else if data[7] & 0b1100 == 0 && data[12..=15].iter().all(|b| *b == 0) {
            // iNES
            format = FileFormat::INes;

            let mapper_number_nybble_2 = data[7] >> 4;

            mapper = (mapper_number_nybble_2 as u16) << 4 | mapper_number_nybble_1 as u16;

            prg_rom = {
                let prg_rom_size = prg_rom_size_lsb as usize * 16384;
                let start = read;
                assert_has_bytes(read + prg_rom_size)?;
                read += prg_rom_size;
                (start, read)
            };

            chr_rom = if chr_rom_size_lsb > 0 {
                let chr_rom_size = chr_rom_size_lsb as usize * 8192;
                let start = read;
                assert_has_bytes(read + chr_rom_size)?;
                read += chr_rom_size;
                Some((start, read))
            } else {
                None
            };
        } else {
            // Archaic iNES
            format = FileFormat::ArchaicINes;
            mapper = mapper_number_nybble_1 as u16;

            prg_rom = {
                let prg_rom_size = prg_rom_size_lsb as usize * 16384;
                let start = read;
                assert_has_bytes(read + prg_rom_size)?;
                read += prg_rom_size;
                (start, read)
            };

            chr_rom = if chr_rom_size_lsb > 0 {
                let chr_rom_size = chr_rom_size_lsb as usize * 8192;
                let start = read;
                assert_has_bytes(read + chr_rom_size)?;
                read += chr_rom_size;
                Some((start, read))
            } else {
                None
            };
        };

        Ok(Self {
            name,
            data,
            format,
            mapper,
            submapper,
            trainer,
            prg_rom,
            chr_rom,
            prg_nvram_size,
            prg_ram_size,
            chr_nvram_size,
            chr_ram_size,
            four_screen_mode,
            battery_present,
            mirroring_vertical,
        })
    }

    pub fn trainer(&self) -> Option<&[u8]> {
        self.trainer.map(|(start, end)| &self.data[start..end])
    }

    pub fn prg_rom(&self) -> &[u8] {
        &self.data[self.prg_rom.0..self.prg_rom.1]
    }

    pub fn chr_rom(&self) -> Option<&[u8]> {
        self.chr_rom.map(|chr_rom| &self.data[chr_rom.0..chr_rom.1])
    }
}
