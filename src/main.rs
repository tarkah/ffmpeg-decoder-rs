use libav_sys::avformat::{
    av_dump_format, av_frame_alloc, av_init_packet, av_read_frame, av_register_all,
    avcodec_find_decoder, avcodec_open2, avcodec_receive_frame, avcodec_send_packet,
    avformat_alloc_context, avformat_close_input, avformat_find_stream_info, avformat_open_input,
    AVCodec, AVDictionary, AVFormatContext, AVFrame, AVPacket, AV_INPUT_BUFFER_PADDING_SIZE,
};
use std::ffi::CString;

const AVCODEC_MAX_AUDIO_FRAME_SIZE: u32 = 192000;

fn main() {
    unsafe {
        av_register_all();

        let mut avf_context: *mut AVFormatContext = avformat_alloc_context();

        let in_file = CString::new("assets/BGM_AI.at3").unwrap();

        let status = avformat_open_input(
            &mut avf_context,
            in_file.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        if status < 0 {
            panic!("Could not open file");
        }

        let status = avformat_find_stream_info(avf_context, std::ptr::null_mut());

        if status < 0 {
            panic!("Could not find stream info");
        }

        av_dump_format(avf_context, 0, in_file.as_ptr(), 0);

        let stream = avf_context
            .as_ref()
            .unwrap()
            .streams
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap();
        let ctx = stream.codec;
        dbg!(ctx.as_ref().unwrap());

        let codec: *mut AVCodec = avcodec_find_decoder(ctx.as_ref().unwrap().codec_id);
        dbg!(codec.as_ref());

        let mut metadata: *mut AVDictionary = avf_context.as_ref().unwrap().metadata;

        avcodec_open2(ctx, codec, &mut metadata);

        let av_packet: *mut AVPacket = {
            let mut uninit_av_packet = std::mem::MaybeUninit::uninit();
            av_init_packet(uninit_av_packet.as_mut_ptr());
            uninit_av_packet.as_mut_ptr()
        };
        dbg!(av_packet);

        let av_frame: *mut AVFrame = av_frame_alloc();
        dbg!(av_frame.as_ref());

        const BUFFER_SIZE: u32 = AVCODEC_MAX_AUDIO_FRAME_SIZE + AV_INPUT_BUFFER_PADDING_SIZE;
        let mut buffer: [u8; BUFFER_SIZE as usize] = [0; BUFFER_SIZE as usize];

        av_packet.as_mut().unwrap().data = buffer.as_mut_ptr();
        av_packet.as_mut().unwrap().size = BUFFER_SIZE as i32;

        dbg!(av_packet.as_ref());

        let mut data: Vec<u8> = vec![];

        let mut i = 0;
        while av_read_frame(avf_context, av_packet) >= 0 {
            avcodec_send_packet(ctx, av_packet);

            while avcodec_receive_frame(ctx, av_frame) == 0 {
                let d = std::slice::from_raw_parts(
                    av_frame
                        .as_ref()
                        .unwrap()
                        .extended_data
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap(),
                    av_frame.as_ref().unwrap().linesize[0] as usize,
                );
                data.extend_from_slice(d);

                i += 1;
            }
        }

        dbg!(i);

        avformat_close_input(&mut avf_context);

        let _ = std::fs::write("assets/BGM_AI.raw", data);
    }
}
