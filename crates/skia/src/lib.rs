//! Skia backend: draws a `Scene::build` frame onto any Skia canvas (a Freya `canvas`, a GPU or raster surface).
//! No depth test: the core's back-to-front order is exact.

use plusrenderer::build::Buffers;
pub use skia_safe as skia;
use skia_safe::{
    vertices::VertexMode, BlendMode, Canvas, Color, FilterMode, Image, Matrix, MipmapMode, Paint,
    Point, SamplingOptions, TileMode, Vertices,
};

/// `textures[slot]` is that slot's image. Runs without one are skipped.
pub fn draw(canvas: &Canvas, buffers: &Buffers, textures: &[Option<Image>]) {
    let sampling = SamplingOptions::new(FilterMode::Nearest, MipmapMode::None);
    let mut indices = Vec::new();

    for run in &buffers.runs {
        let Some(Some(image)) = textures.get(run.texture as usize) else {
            continue;
        };

        // UVs are 0..1. Scale the shader so they index the whole image.
        let to_uv = Matrix::scale((1.0 / image.width() as f32, 1.0 / image.height() as f32));
        let Some(shader) = image.to_shader((TileMode::Clamp, TileMode::Clamp), sampling, &to_uv)
        else {
            continue;
        };

        // The core emits each run's vertices contiguously, so copy only that range and rebase.
        let run_indices = &buffers.indices[run.start as usize..(run.start + run.count) as usize];
        let (Some(&lo), Some(&hi)) = (run_indices.iter().min(), run_indices.iter().max()) else {
            continue;
        };
        let (lo, hi) = (lo as usize, hi as usize + 1);
        indices.clear();
        indices.extend(run_indices.iter().map(|&i| i - lo as u16));

        let points = |v: &[f32]| -> Vec<Point> {
            v[lo * 2..hi * 2]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&[x, y]| Point::new(x, y))
                .collect()
        };
        let colors: Vec<Color> = buffers.colors[lo..hi]
            .iter()
            .map(|&c| Color::new(c))
            .collect();
        let vertices = Vertices::new_copy(
            VertexMode::Triangles,
            &points(&buffers.positions),
            &points(&buffers.uvs),
            &colors,
            Some(&indices),
        );

        let mut paint = Paint::default();
        paint.set_shader(shader);
        // Texel times the baked light, as the WebGL shader does.
        canvas.draw_vertices(&vertices, BlendMode::Modulate, &paint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plusrenderer::{Arms, Camera, Scene};
    use skia_safe::{surfaces, ImageInfo};

    #[test]
    fn draws_the_player_with_its_skin_shaded() {
        let skin = include_bytes!("../../../packages/demo/public/skin.png");
        let skin = Image::from_encoded(skia_safe::Data::new_copy(skin)).expect("skin decodes");
        let mut scene = Scene::default();
        scene.load_player(0, Arms::Wide);
        scene.pose(0.0);
        let buffers = scene.build(
            &Camera::orbit(0.0, 8.0, 70.0, [0.0, 18.0, 0.0]),
            [128.0, 128.0],
        );

        let mut surface = surfaces::raster_n32_premul((128, 128)).unwrap();
        draw(surface.canvas(), buffers, &[Some(skin)]);
        let mut pixels = vec![0u8; 128 * 128 * 4];
        let info = ImageInfo::new_n32_premul((128, 128), None);
        assert!(surface.read_pixels(&info, &mut pixels, 128 * 4, (0, 0)));

        let opaque = pixels.chunks(4).filter(|p| p[3] == 255).count();
        assert!(
            opaque > 128 * 128 / 20,
            "the player covers the middle, got {opaque} px"
        );
        assert_eq!(pixels[3], 0, "corners stay clear");
    }
}
