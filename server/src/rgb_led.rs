// Adapted from:
// https://github.com/fkohlgrueber/esp32c3-idf-led-example

use esp_idf_sys::*;

const WS2812_T0H_NS: u32 = 350;
const WS2812_T0L_NS: u32 = 1000;
const WS2812_T1H_NS: u32 = 1000;
const WS2812_T1L_NS: u32 = 350;

#[derive(Debug)]
pub struct Led {
    ws2812_t0h_ticks: u32,
    ws2812_t0l_ticks: u32,
    ws2812_t1h_ticks: u32,
    ws2812_t1l_ticks: u32,
    buffer: [u8; 3],
    channel: rmt_channel_t,
}

impl Led {
    pub fn new(channel: rmt_channel_t, gpio_num: gpio_num_t) -> Result<Self, EspError> {
        let config = rmt_config_t {
            rmt_mode: rmt_mode_t_RMT_MODE_TX,
            gpio_num,
            channel,
            clk_div: 2,
            mem_block_num: 1,
            flags: 0,
            __bindgen_anon_1: rmt_config_t__bindgen_ty_1 {
                tx_config: rmt_tx_config_t {
                    carrier_freq_hz: 38000,
                    carrier_level: rmt_carrier_level_t_RMT_CARRIER_LEVEL_HIGH,
                    idle_level: rmt_carrier_level_t_RMT_CARRIER_LEVEL_LOW,
                    carrier_duty_percent: 33,
                    carrier_en: false,
                    loop_en: false,
                    idle_output_en: true,
                    loop_count: 0,
                },
            },
        };

        unsafe {
            esp_result!(rmt_config(&config as *const rmt_config_t), ())?;
            esp_result!(rmt_driver_install(config.channel, 0, 0), ())?;
        }

        let mut counter_clk_hz: u32 = 0;
        unsafe {
            esp_result!(
                rmt_get_counter_clock(config.channel, &mut counter_clk_hz),
                ()
            )?;
        }
        let ratio = counter_clk_hz as f32 / 1e9;

        unsafe {
            esp_result!(rmt_translator_init(config.channel, Some(Self::adapter)), ())?;
        }

        let ws2812_t0h_ticks = (ratio * WS2812_T0H_NS as f32) as u32;
        let ws2812_t0l_ticks = (ratio * WS2812_T0L_NS as f32) as u32;
        let ws2812_t1h_ticks = (ratio * WS2812_T1H_NS as f32) as u32;
        let ws2812_t1l_ticks = (ratio * WS2812_T1L_NS as f32) as u32;

        let led = Led {
            ws2812_t0h_ticks,
            ws2812_t0l_ticks,
            ws2812_t1h_ticks,
            ws2812_t1l_ticks,
            buffer: [0, 0, 0],
            channel,
        };

        Ok(led)
    }

    pub fn set_color(&mut self, red: u8, green: u8, blue: u8) -> Result<(), EspError> {
        self.buffer = [green, red, blue];

        unsafe {
            esp_result!(
                rmt_write_sample(self.channel, (self as *mut Self).cast(), 1 as u32 * 3, true,),
                ()
            )?;
            esp_result!(rmt_wait_tx_done(self.channel, 1_000_000), ())?;
        }

        Ok(())
    }

    unsafe extern "C" fn adapter(
        src: *const c_types::c_void,
        dest: *mut rmt_item32_t,
        src_size: u32,
        wanted_num: u32,
        translated_size: *mut u32,
        item_num: *mut u32,
    ) {
        if src.is_null() || dest.is_null() {
            *translated_size = 0;
            *item_num = 0;

            return;
        }

        let led_strip_p: *const Self = src.cast();
        let led_strip_ref: &Self = &*led_strip_p;

        let bit0 = Self::get_rmt_item32(
            led_strip_ref.ws2812_t0h_ticks,
            1,
            led_strip_ref.ws2812_t0l_ticks,
            0,
        );
        let bit1 = Self::get_rmt_item32(
            led_strip_ref.ws2812_t1h_ticks,
            1,
            led_strip_ref.ws2812_t1l_ticks,
            0,
        );

        let mut size = 0;
        let mut num = 0;
        let mut psrc: *const u8 = led_strip_ref.buffer.as_ptr().cast();
        let mut pdest = dest;

        while size < src_size && num < wanted_num {
            for i in 0..8 {
                // MSB first
                if *psrc & (1 << (7 - i)) != 0 {
                    (*pdest) = bit1;
                } else {
                    (*pdest) = bit0;
                }
                num += 1;
                pdest = pdest.add(1);
            }

            size += 1;
            psrc = psrc.add(1);
        }

        *translated_size = size;
        *item_num = num;
    }

    fn get_rmt_item32(duration0: u32, level0: u32, duration1: u32, level1: u32) -> rmt_item32_t {
        let mut item = rmt_item32_t__bindgen_ty_1__bindgen_ty_1::default();
        item.set_duration0(duration0);
        item.set_duration1(duration1);
        item.set_level0(level0);
        item.set_level1(level1);

        rmt_item32_t {
            __bindgen_anon_1: rmt_item32_t__bindgen_ty_1 {
                __bindgen_anon_1: item,
            },
        }
    }
}

impl Drop for Led {
    fn drop(&mut self) {
        unsafe {
            rmt_driver_uninstall(self.channel);
        }
    }
}
