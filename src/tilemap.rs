use crate::{Buffer, BufferMut, Color, TileId, Tileset};

use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};
use fast_srgb8::{f32x4_to_srgb8, srgb8_to_f32};
use simple_blit::{blit_with, BlitOptions};

/// Tile in a [`Tilemap`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tile<U = ()> {
    /// Id of the tile.
    pub id: TileId,

    /// Color of the tile.
    /// Will be multiplied by the tile's 'original' color (the one in the tileset).
    pub color: Color,

    /// Blit options.
    pub opts: BlitOptions,

    /// Custom user data.
    pub user_data: U,
}

impl<U> Tile<U>
where
    U: Default,
{
    /// Contruct a new tile.
    #[inline]
    pub fn new(id: TileId) -> Self {
        Self {
            id,
            color: Color::new(255, 255, 255, 255),
            opts: BlitOptions::None,
            user_data: U::default(),
        }
    }
}

impl<U> Tile<U> {
    /// Set tile color.
    ///
    /// Allows for chaining tile creation like `Tile::new(4).with_color(Color::new(255, 0, 255, 255)).solid()` etc.
    #[inline]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set tile color.
    ///
    /// Allows for chaining tile modifying like `tilemap.get_mut_tile(1, 1).unwrap().set_color(Color::new(255, 0, 255, 255))` etc.
    #[inline]
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Set tile user data.
    ///
    /// Allows for chaining tile creation like `Tile::new(4).with_color(Color::new(255, 0, 255, 255))` etc.
    #[inline]
    pub fn with_user_data(mut self, user_data: U) -> Self {
        self.user_data = user_data;
        self
    }

    /// Set tile user data.
    ///
    /// Allows for chaining tile modifying like `tilemap.get_mut_tile(1, 1).unwrap().set_color(Color::new(255, 0, 255, 255))` etc.
    #[inline]
    pub fn set_user_data(&mut self, user_data: U) -> &mut Self {
        self.user_data = user_data;
        self
    }

    /// Specify the blit options.
    ///
    /// Allows for chaining tile creation like `Tile::new(4).with_color(Color::new(255, 0, 255, 255))` etc.
    #[inline]
    pub fn with_blit_options(mut self, opts: BlitOptions) -> Self {
        self.opts = opts;
        self
    }

    /// Set the tile's blit options.
    ///
    /// Allows for chaining tile modifying like `tilemap.get_mut_tile(1, 1).unwrap().set_color(Color::new(255, 0, 255, 255))` etc.
    #[inline]
    pub fn set_blit_options(&mut self, opts: BlitOptions) -> &mut Self {
        self.opts = opts;
        self
    }
}

/// A map that holds a tileset and a collection of tiles.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tilemap<C, U = ()> {
    tileset: Tileset<C>,
    tiles: Vec<Tile<U>>,
    width: u32,
    height: u32,
}

impl<C, U> Tilemap<C, U>
where
    U: Default + Clone,
{
    /// Construct a new tilemap.
    /// `width` and `height` are map's size in tiles.
    #[inline]
    pub fn new(width: u32, height: u32, tileset: Tileset<C>) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::default(); (width * height) as usize],
            tileset,
        }
    }
}

impl<C, U> Tilemap<C, U> {
    /// Map's width in tiles.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Map's height in tiles.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Tileset used by this map.
    #[inline]
    pub fn tileset(&self) -> &Tileset<C> {
        &self.tileset
    }

    /// Map's tiles.
    #[inline]
    pub fn tiles(&self) -> &[Tile<U>] {
        &self.tiles
    }

    /// Map's tiles (mutable).
    #[inline]
    pub fn tiles_mut(&mut self) -> &mut [Tile<U>] {
        &mut self.tiles
    }

    /// Get a tile at (x, y).
    #[inline]
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile<U>> {
        self.tiles.get((y * self.width + x) as usize)
    }

    /// Get a mutable ref to a tile at (x, y).
    #[inline]
    pub fn get_mut_tile(&mut self, x: u32, y: u32) -> Option<&mut Tile<U>> {
        self.tiles.get_mut((y * self.width + x) as usize)
    }

    /// Set a tile at (x, y).
    #[inline]
    pub fn set_tile(&mut self, x: u32, y: u32, tile: Tile<U>) {
        if let Some(t) = self.tiles.get_mut((y * self.width + x) as usize) {
            *t = tile;
        }
    }
}

impl<C> Tilemap<C>
where
    C: AsRef<[u8]>,
{
    /// Render the map onto a buffer at pixel offset `(offset_x, offset_y)`.
    pub fn render(
        &self,
        surface: &mut (impl BufferMut<Color> + ?Sized),
        offset_x: i32,
        offset_y: i32,
    ) {
        for ty in 0..self.height {
            for tx in 0..self.width {
                let &Tile {
                    id: tile,
                    color,
                    opts,
                    ..
                } = self.get(tx, ty);

                if let Some((x, y)) = self.tileset.get_tile_pos(tile) {
                    blit_with(
                        surface,
                        (offset_x, offset_y),
                        &self.tileset,
                        (x as _, y as _),
                        self.tileset.opts.tile_size,
                        opts,
                        |dest, src, _| {
                            if Some(*src) != self.tileset.opts.key_color {
                                let [r, g, b, a] = f32x4_to_srgb8([
                                    srgb8_to_f32(src.r) * srgb8_to_f32(color.r),
                                    srgb8_to_f32(src.g) * srgb8_to_f32(color.g),
                                    srgb8_to_f32(src.b) * srgb8_to_f32(color.b),
                                    srgb8_to_f32(src.a) * srgb8_to_f32(color.a),
                                ]);

                                *dest = Color::new(r, g, b, a);
                            }
                        },
                    );
                }
            }
        }
    }
}

impl<C> Buffer<Tile> for Tilemap<C> {
    #[inline]
    fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    fn get(&self, x: u32, y: u32) -> &Tile {
        self.tiles.index((y * self.width + x) as usize)
    }
}

impl<C> BufferMut<Tile> for Tilemap<C> {
    #[inline]
    fn get_mut(&mut self, x: u32, y: u32) -> &mut Tile {
        self.tiles.index_mut((y * self.width + x) as usize)
    }
}
