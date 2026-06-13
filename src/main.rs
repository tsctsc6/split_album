use std::{
    fs::remove_file,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Ok};
use clap::Parser;
use rcue::parser::parse_from_file;
use xshell::Shell;

pub mod command;

fn main() -> anyhow::Result<()> {
    let cli = command::Cli::parse();
    println!("{:?}", cli);
    let sh = Shell::new()?;

    // Parse the file (lenient mode set to true)
    let cue = parse_from_file(&cli.file, true)?;
    let cue_location = Path::new(&cli.file);
    let cue_location = cue_location.parent().unwrap().to_string_lossy();
    let cue_location = PathBuf::from(cue_location.to_string());
    let out_dir = PathBuf::from(&cli.out_dir);
    let cover_file = if cli.without_cover {
        None
    } else {
        let cover_file = cue_location.join("cover.jpg").to_string_lossy().to_string();
        Some(cover_file)
    };
    let performer = cue.performer.context("Read cue performer failed")?;

    for file in cue.files.iter() {
        let album_file = cue_location.join(&file.file).to_string_lossy().to_string();
        let tracks_iter = file.tracks.iter();
        let tracks_iter2 = file
            .tracks
            .iter()
            .skip(1)
            .map(Some)
            .chain(std::iter::once(None));

        for (current_track, next_track) in tracks_iter.zip(tracks_iter2) {
            let start_cut_point = get_cut_point(&current_track.indices)?;
            let end_cut_point = if let Some(next) = next_track {
                get_cut_point(&next.indices)?
            } else {
                get_last_cut_point(&sh, &album_file)?
            };

            split(
                &sh,
                &album_file,
                &out_dir,
                cue.title.as_ref().context("Read album_title failed")?,
                current_track
                    .title
                    .as_ref()
                    .context("Read track_title failed")?,
                current_track.performer.as_ref().unwrap_or(&performer),
                &current_track.no,
                start_cut_point,
                end_cut_point,
                cover_file.clone(),
                &cli.ext_args,
            )?;
        }
    }
    Ok(())
}

fn get_cut_point(track_indices: &Vec<(String, Duration)>) -> anyhow::Result<Duration> {
    let cut_point = match track_indices.len() {
        1 => track_indices[0].1,
        2 => (track_indices[0].1 + track_indices[1].1) / 2,
        _ => Err(anyhow::anyhow!("get_cut_point failed"))?,
    };
    Ok(cut_point)
}

fn get_last_cut_point(sh: &Shell, album_file: &str) -> anyhow::Result<Duration> {
    let mut args: Vec<String> = vec![
        "-v".into(),
        "error".into(),
        "-show_entries".into(),
        "format=duration".into(),
        "-of".into(),
        "default=noprint_wrappers=1:nokey=1".into(),
    ];
    args.push(album_file.into());
    let ffprobe_output = sh.cmd("ffprobe").args(&args).read()?;
    let duration = Duration::from_secs_f64(ffprobe_output.parse()?);
    Ok(duration)
}

fn split(
    sh: &Shell,
    album_file: &str,
    out_dir: &PathBuf,
    album_title: &str,
    track_title: &str,
    performer: &str,
    track_number: &str,
    start_cut_point: Duration,
    end_cut_point: Duration,
    cover_file: Option<String>,
    ext_args: &Vec<String>,
) -> anyhow::Result<()> {
    let out_file = out_dir.join(format!("{track_number} {track_title}.flac"));
    let out_file = out_file.to_string_lossy().to_string();
    let path = Path::new(&out_file);
    if path.exists() {
        remove_file(&out_file)?;
    }
    let start_time = start_cut_point.as_secs_f64().to_string();
    let end_time = end_cut_point.as_secs_f64().to_string();

    let mut args: Vec<String> = vec!["-v".into(), "error".into()];
    args.push("-i".into());
    args.push(album_file.to_string());

    if let Some(cover_file) = cover_file.as_ref() {
        args.push("-i".into());
        args.push(cover_file.to_string());
    }

    args.push("-map".into());
    args.push("0:a".into());
    args.push("-ss".into());
    args.push(start_time);
    args.push("-to".into());
    args.push(end_time.into());
    args.push("-c:a".into());
    args.push("flac".into());

    if let Some(_) = cover_file {
        args.push("-map".into());
        args.push("1:v".into());
        args.push("-c:v".into());
        args.push("copy".into());
        args.push("-disposition:v:0".into());
        args.push("attached_pic".into());
    }

    args.push("-metadata".into());
    args.push(format!("album={album_title}"));
    args.push("-metadata".into());
    args.push(format!("title={track_title}"));
    args.push("-metadata".into());
    args.push(format!("artist={performer}"));
    args.push("-metadata".into());
    args.push(format!("track={track_number}"));

    for ext_arg in ext_args {
        args.push(ext_arg.clone());
    }

    args.push(out_file);

    println!("{:?}", &args);
    let ffmpeg_output = sh.cmd("ffmpeg").args(&args).read()?;
    println!("{ffmpeg_output}");
    Ok(())
}
