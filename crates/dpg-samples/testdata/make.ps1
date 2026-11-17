# Regenerates the test tones: 0.5 s of 440 Hz at half scale, made from a
# formula so no recorded audio is involved. Needs ffmpeg on the PATH.
$wave = "0.5*sin(2*PI*440*t)"
$tone = "aevalsrc=${wave}:s={0}:d=0.5"
# Two equal channels: letting ffmpeg upmix mono would lower the level 3 dB.
$stereo = "aevalsrc=${wave}|${wave}:s={0}:d=0.5"
$clean = @("-y", "-hide_banner", "-loglevel", "error", "-map_metadata", "-1", "-fflags", "+bitexact")
ffmpeg -f lavfi -i ($tone -f 44100) @clean -c:a flac tone-44k-mono.flac
ffmpeg -f lavfi -i ($stereo -f 44100) @clean -c:a libmp3lame -b:a 128k tone-44k-stereo.mp3
ffmpeg -f lavfi -i ($tone -f 48000) @clean -c:a aac -b:a 96k tone-48k-mono.m4a
