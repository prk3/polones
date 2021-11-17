pub struct GameFile {
    data: Vec<u8>,
    pub format: FileFormat,
    pub mapper: u16,
    trainer: Option<(usize, usize)>,
    prg_rom: (usize, usize),
    chr_rom: (usize, usize),
    misc_rom: Option<(usize, usize)>,
}

#[derive(Debug)]
pub enum FileFormat {
    ArchaicINes,
    INes,
    Nes20,
}

impl GameFile {
    pub fn read(data: Vec<u8>) -> Result<Self, ()> {
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
        let hard_wired_four_screen_mode = data[6] & 0b1000 > 0;
        let trainer_present = data[6] & 0b0100 > 0;
        let battery_present = data[6] & 0b0010 > 0;
        let hard_wired_nametable_mirroring_vertical = data[6] & 0b0001 > 0;

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
        let prg_size = if prg_rom_size_msb < 0xf {
            ((prg_rom_size_msb as usize) << 8 | prg_rom_size_lsb as usize) * 16384
        } else {
            let multiplier = (prg_rom_size_lsb & 0b00000011) * 2 + 1;
            let exponent = prg_rom_size_lsb >> 2;
            2usize.pow(exponent as u32) * multiplier as usize
        };

        let chr_size = if chr_rom_size_msb < 0xf {
            ((chr_rom_size_msb as usize) << 8 | chr_rom_size_lsb as usize) * 8192
        } else {
            let multiplier = (chr_rom_size_lsb & 0b00000011) * 2 + 1;
            let exponent = chr_rom_size_lsb >> 2;
            2usize.pow(exponent as u32) * multiplier as usize
        };

        let format: FileFormat;
        let prg_rom: (usize, usize);
        let chr_rom: (usize, usize);
        let mapper: u16;
        let mut misc_rom: Option<(usize, usize)> = None;

        // Now we have enough data to decide which format we're dealing with.
        // If it's not NES 2.0, we'll reinterpret byte 9.
        if data[7] & 0b00001100 == 0b00001000
            && data.len() >= (16 + (trainer_present as usize * 512) + prg_size + chr_size)
        {
            // NES 2.0
            format = FileFormat::Nes20;

            let mapper_number_nybble_2 = data[7] >> 4;
            let console_type = data[7] & 0b00000011;
            let submapper_number = data[8] >> 4;
            let mapper_number_nybble_3 = data[8] & 0b00001111;

            let non_volatile_shift_count = data[10] >> 4;
            let volatile_shift_count = data[10] & 0b00001111;

            let chr_ram_size_shift = data[11] >> 4;
            let chr_nvram_size_shift = data[11] & 0b00001111;
            let cpu_ppu_timing_mode = data[12] & 0b00000011;

            let hardware_type = if console_type == 1 {
                Some(data[13] >> 4)
            } else {
                None
            };
            let ppu_type = if console_type == 1 {
                Some(data[13] & 0b00001111)
            } else {
                None
            };
            let extended_console_type = if console_type == 3 {
                Some(data[13] & 0b00001111)
            } else {
                None
            };

            let miscellaneous_roms_number = data[14] & 0b00000011;
            let default_expansion_device = data[15] & 0b00111111;

            mapper = (mapper_number_nybble_3 as u16) << 8
                | (mapper_number_nybble_2 as u16) << 4
                | mapper_number_nybble_1 as u16;

            prg_rom = {
                let start = read;
                assert_has_bytes(read + prg_size)?;
                read += prg_size;
                (start, read)
            };

            chr_rom = {
                let start = read;
                assert_has_bytes(read + chr_size)?;
                read += chr_size;
                (start, read)
            };

            misc_rom = if miscellaneous_roms_number > 0 {
                let start = read;
                let size = data.len() - read;
                read += size;
                Some((start, read))
            } else {
                None
            }
        } else if data[7] & 0b00001100 == 0b00000000 && data[12..=15].iter().all(|b| *b == 0) {
            // iNES
            format = FileFormat::INes;

            let mapper_number_nybble_2 = data[7] >> 4;

            mapper = (mapper_number_nybble_2 as u16) << 4 | mapper_number_nybble_2 as u16;

            prg_rom = {
                let prg_rom_size = prg_rom_size_lsb as usize * 16384;
                let start = read;
                assert_has_bytes(read + prg_rom_size)?;
                read += prg_rom_size;
                (start, read)
            };

            chr_rom = {
                let chr_rom_size = chr_rom_size_lsb as usize * 8192;
                let start = read;
                assert_has_bytes(read + chr_rom_size)?;
                read += chr_rom_size;
                (start, read)
            };

            // TODO read other flags too
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

            chr_rom = {
                let chr_rom_size = chr_rom_size_lsb as usize * 8192;
                let start = read;
                assert_has_bytes(read + chr_rom_size)?;
                read += chr_rom_size;
                (start, read)
            };
        };

        Ok(Self {
            data,
            format,
            mapper,
            trainer,
            prg_rom,
            chr_rom,
            misc_rom,
        })
    }

    pub fn trainer(&self) -> Option<&[u8]> {
        self.trainer.map(|(start, end)| &self.data[start..end])
    }

    pub fn prg_rom(&self) -> &[u8] {
        &self.data[self.prg_rom.0..self.prg_rom.1]
    }

    pub fn chr_rom(&self) -> &[u8] {
        &self.data[self.chr_rom.0..self.chr_rom.1]
    }

    pub fn misc_rom(&self) -> Option<&[u8]> {
        self.misc_rom.map(|(start, end)| &self.data[start..end])
    }
}
