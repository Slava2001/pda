pub enum Func {
    FmReceive,
    QueryLibraryID,
}

impl Func {
    fn bits(self) -> u8 {
        match self {
            Self::FmReceive => 0x00,
            Self::QueryLibraryID => 0x0F,
        }
    }
}

pub enum OpMode {
    AnalogAudioOutputs,
    DigitalAudioOutput,
    DigitalAudioOutputs,
    AnalogAndDigitalAudioOutputs,
}

impl OpMode {
    fn bits(self) -> u8 {
        match self {
            Self::AnalogAudioOutputs => 0x05,
            Self::DigitalAudioOutput => 0x0B,
            Self::DigitalAudioOutputs => 0xB0,
            Self::AnalogAndDigitalAudioOutputs => 0xB5,
        }
    }
}

pub enum Property {
    GpoIen,
    DigitalOutputFormat,
    DigitalOutputSampleRate,
    RefclkFreq,
    RefclkPrescale,
    FmDeemphasis,
    FmChannelFilter,
    FmBlendStereoThreshold,
    FmBlendMonoThreshold,
    FmAntennaInput,
    FmMaxTuneError,
    FmRsqIntSource,
    FmRsqSnrHiThreshold,
    FmRsqSnrLoThreshold,
    FmRsqRssiHiThreshold,
    FmRsqRssiLoThreshold,
    FmRsqMultipathHiThreshold,
    FmRsqMultipathLoThreshold,
    FmRsqBlendThreshold,
    FmSoftMuteRate,
    FmSoftMuteSlope,
    FmSoftMuteMaxAttenuation,
    FmSoftMuteSnrThreshold,
    FmSoftMuteReleaseRate,
    FmSoftMuteAttackRate,
    FmSeekBandBottom,
    FmSeekBandTop,
    FmSeekFreqSpacing,
    FmSeekTuneSnrThreshold,
    FmSeekTuneRssiTreshold,
    FmRdsIntSource,
    FmRdsIntFifoCount,
    FmRdsConfig,
    FmRdsConfidence,
    FmBlendRssiStereoThreshold,
    FmBlendRssiMonoThreshold,
    FmBlendRssiAttackRate,
    FmBlendRssiReleaseRate,
    FmBlendSnrStereoThreshold,
    FmBlendSnrMonoThreshold,
    FmBlendSnrAttackRate,
    FmBlendSnrReleaseRate,
    FmBlendMultipathStereoThreshold,
    FmBlendMultipathMonoThreshold,
    FmBlendMultipathAttackRate,
    FmBlendMultipathReleaseRate,
    FmHicutSnrHighThreshold,
    FmHicutSnrLowThreshold,
    FmHicutAttackRate,
    FmHicutReleaseRate,
    FmHicutMultipathTriggerThreshold,
    FmHicutMultipathEndThreshold,
    FmHicutCutoffFrequency,
    RxVolume,
    RxHardMute,
}

impl Property {
    fn to_be_bytes(&self) -> [u8; 2] {
        match self {
            Self::GpoIen => [0x00, 0x01],
            Self::DigitalOutputFormat => [0x01, 0x02],
            Self::DigitalOutputSampleRate => [0x01, 0x04],
            Self::RefclkFreq => [0x02, 0x01],
            Self::RefclkPrescale => [0x02, 0x02],
            Self::FmDeemphasis => [0x11, 0x00],
            Self::FmChannelFilter => [0x11, 0x02],
            Self::FmBlendStereoThreshold => [0x11, 0x05],
            Self::FmBlendMonoThreshold => [0x11, 0x06],
            Self::FmAntennaInput => [0x11, 0x07],
            Self::FmMaxTuneError => [0x11, 0x08],
            Self::FmRsqIntSource => [0x12, 0x00],
            Self::FmRsqSnrHiThreshold => [0x12, 0x01],
            Self::FmRsqSnrLoThreshold => [0x12, 0x02],
            Self::FmRsqRssiHiThreshold => [0x12, 0x03],
            Self::FmRsqRssiLoThreshold => [0x12, 0x04],
            Self::FmRsqMultipathHiThreshold => [0x12, 0x05],
            Self::FmRsqMultipathLoThreshold => [0x12, 0x06],
            Self::FmRsqBlendThreshold => [0x12, 0x07],
            Self::FmSoftMuteRate => [0x13, 0x00],
            Self::FmSoftMuteSlope => [0x13, 0x01],
            Self::FmSoftMuteMaxAttenuation => [0x13, 0x02],
            Self::FmSoftMuteSnrThreshold => [0x13, 0x03],
            Self::FmSoftMuteReleaseRate => [0x13, 0x04],
            Self::FmSoftMuteAttackRate => [0x13, 0x05],
            Self::FmSeekBandBottom => [0x14, 0x00],
            Self::FmSeekBandTop => [0x14, 0x01],
            Self::FmSeekFreqSpacing => [0x14, 0x02],
            Self::FmSeekTuneSnrThreshold => [0x14, 0x03],
            Self::FmSeekTuneRssiTreshold => [0x14, 0x04],
            Self::FmRdsIntSource => [0x15, 0x00],
            Self::FmRdsIntFifoCount => [0x15, 0x01],
            Self::FmRdsConfig => [0x15, 0x02],
            Self::FmRdsConfidence => [0x15, 0x03],
            Self::FmBlendRssiStereoThreshold => [0x18, 0x00],
            Self::FmBlendRssiMonoThreshold => [0x18, 0x01],
            Self::FmBlendRssiAttackRate => [0x18, 0x02],
            Self::FmBlendRssiReleaseRate => [0x18, 0x03],
            Self::FmBlendSnrStereoThreshold => [0x18, 0x04],
            Self::FmBlendSnrMonoThreshold => [0x18, 0x05],
            Self::FmBlendSnrAttackRate => [0x18, 0x06],
            Self::FmBlendSnrReleaseRate => [0x18, 0x07],
            Self::FmBlendMultipathStereoThreshold => [0x18, 0x08],
            Self::FmBlendMultipathMonoThreshold => [0x18, 0x09],
            Self::FmBlendMultipathAttackRate => [0x18, 0x0A],
            Self::FmBlendMultipathReleaseRate => [0x18, 0x0B],
            Self::FmHicutSnrHighThreshold => [0x1A, 0x00],
            Self::FmHicutSnrLowThreshold => [0x1A, 0x01],
            Self::FmHicutAttackRate => [0x1A, 0x02],
            Self::FmHicutReleaseRate => [0x1A, 0x03],
            Self::FmHicutMultipathTriggerThreshold => [0x1A, 0x04],
            Self::FmHicutMultipathEndThreshold => [0x1A, 0x05],
            Self::FmHicutCutoffFrequency => [0x1A, 0x06],
            Self::RxVolume => [0x40, 0x00],
            Self::RxHardMute => [0x40, 0x01],
        }
    }
}

pub enum Cmd {
    PowerUp {
        cts_interrupt_enable: bool,
        gpo2_output_enable: bool,
        patch_enable: bool,
        crystal_oscillator_enable: bool,
        func: Func,
        opmode: OpMode,
    },
    GetRev,
    PowerDown,
    SetProperty {
        prop: Property,
        val: u16,
    },
    GetProperty {
        prop: Property,
    },
    GetIntStatus,
    FmTuneFreq {
        freeze: bool,
        fast: bool,
        freq: u16,
        antcap: u8,
    },
    FmSeekStart {
        seek_up: bool,
        wrap: bool,
    },
    FmTuneStatus {
        cancel: bool,
        int_ack: bool,
    },
}

impl Cmd {
    pub fn encode(self, buf: &mut [u8; 8]) -> usize {
        match self {
            Self::PowerUp {
                cts_interrupt_enable,
                gpo2_output_enable,
                patch_enable,
                crystal_oscillator_enable,
                func,
                opmode,
            } => {
                buf[0] = 0x01;
                buf[1] = (u8::from(cts_interrupt_enable) << 7)
                    | (u8::from(gpo2_output_enable) << 6)
                    | (u8::from(patch_enable) << 5)
                    | (u8::from(crystal_oscillator_enable) << 4)
                    | func.bits();
                buf[2] = opmode.bits();
                3
            }
            Self::GetRev => {
                buf[0] = 0x10;
                1
            }
            Self::PowerDown => {
                buf[0] = 0x11;
                1
            }
            Self::SetProperty { prop, val } => {
                buf[0] = 0x12;
                buf[1] = 0x00;
                buf[2..=3].copy_from_slice(&prop.to_be_bytes());
                buf[4..=5].copy_from_slice(&val.to_be_bytes());
                6
            }
            Self::GetProperty { prop } => {
                buf[0] = 0x13;
                buf[1] = 0x00;
                buf[2..=3].copy_from_slice(&prop.to_be_bytes());
                4
            }
            Self::GetIntStatus => {
                buf[0] = 0x14;
                1
            }
            Self::FmTuneFreq {
                freeze,
                fast,
                freq,
                antcap,
            } => {
                buf[0] = 0x20;
                buf[1] = u8::from(freeze) << 1 | u8::from(fast);
                buf[2..=3].copy_from_slice(&freq.to_be_bytes());
                buf[4] = antcap;
                5
            }
            Self::FmSeekStart { seek_up, wrap } => {
                buf[0] = 0x21;
                buf[1] = u8::from(seek_up) << 3 | u8::from(wrap) << 2;
                2
            }
            Self::FmTuneStatus { cancel, int_ack } => {
                buf[0] = 0x22;
                buf[1] = u8::from(cancel) << 1 | u8::from(int_ack);
                2
            }
        }
    }
}

pub trait Response<const L: usize> {
    const LEN: usize = L;
    fn form_bytes(bytes: &[u8; L]) -> Self;
}

#[derive(Debug)]
pub struct Status {
    pub clear_to_send: bool,
    pub err: bool,
    d5: bool,
    d4: bool,
    pub rsq_int: bool,
    pub rds_int: bool,
    pub asq_int: bool,
    pub stc_int: bool,
}

impl Response<1> for Status {
    fn form_bytes(bytes: &[u8; 1]) -> Self {
        Self {
            clear_to_send: (bytes[0] & (1 << 7)) != 0,
            err: (bytes[0] & (1 << 6)) != 0,
            d5: (bytes[0] & (1 << 5)) != 0,
            d4: (bytes[0] & (1 << 4)) != 0,
            rsq_int: (bytes[0] & (1 << 3)) != 0,
            rds_int: (bytes[0] & (1 << 2)) != 0,
            asq_int: (bytes[0] & (1 << 1)) != 0,
            stc_int: (bytes[0] & (1 << 0)) != 0,
        }
    }
}

pub struct LibraryID {
    pub status: Status,
    pub part_number: u8,
    pub firmware: [char; 2],
    pub chip_revision: char,
    pub library_id: u8,
}

impl Response<8> for LibraryID {
    fn form_bytes(bytes: &[u8; 8]) -> Self {
        Self {
            status: Status::form_bytes(&[bytes[0]]),
            part_number: bytes[1],
            firmware: [bytes[2].into(), bytes[3].into()],
            chip_revision: bytes[6].into(),
            library_id: bytes[7],
        }
    }
}

#[derive(Debug)]
pub struct TuneStatus {
    pub status: Status,
    pub band_limit: bool,
    pub afc_rail_indicator: bool,
    pub valid: bool,
    pub freq: u16,
    pub rssi: u8,
    pub snr: u8,
    pub multipath: u8,
    pub read_ant_capacitor: u8,
}

impl Response<8> for TuneStatus {
    fn form_bytes(bytes: &[u8; 8]) -> Self {
        Self {
            status: Status::form_bytes(&[bytes[0]]),
            band_limit: (bytes[1] & (1 << 7)) != 0,
            afc_rail_indicator: (bytes[1] & (1 << 1)) != 0,
            valid: (bytes[1] & (1 << 0)) != 0,
            freq: (u16::from(bytes[2]) << 8) | u16::from(bytes[3]),
            rssi: bytes[4],
            snr: bytes[5],
            multipath: bytes[6],
            read_ant_capacitor: bytes[7],
        }
    }
}
