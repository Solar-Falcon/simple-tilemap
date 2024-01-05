use crate::{Buffer, BufferMut, Color};

use core::mem::size_of;
use core::ops::{Index, IndexMut};
use rgb::AsPixels;

/// Id of a tile in a tileset.
/// Tiles in a tileset are counted left-to-right then top-to-bottom.
pub type TileId = u32;

/// Options used when creating a tileset.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TilesetOptions {
    /// Size (width x height) of a single tile.
    pub tile_size: (u32, u32),
    /// Offset (x, y) - first tile's top left corner.
    pub offset: (u32, u32),
    /// Margin (x, y) - distance between tiles.
    pub margin: (u32, u32),
    /// Key color aka mask/background color. Gets ignored when rendering the tile.
    pub key_color: Option<Color>,
}

impl TilesetOptions {
    /// Create a new TilesetOptions.
    #[inline]
    pub const fn new(tile_width: u32, tile_height: u32) -> Self {
        Self {
            tile_size: (tile_width, tile_height),
            offset: (0, 0),
            margin: (0, 0),
            key_color: None,
        }
    }

    /// Specify offset.
    #[inline]
    pub const fn with_offset(mut self, offset_x: u32, offset_y: u32) -> Self {
        self.offset = (offset_x, offset_y);
        self
    }

    /// Specify margin.
    #[inline]
    pub const fn with_margin(mut self, margin_x: u32, margin_y: u32) -> Self {
        self.margin = (margin_x, margin_y);
        self
    }

    /// Specify key color.
    #[inline]
    pub const fn with_key_color(mut self, key_color: Color) -> Self {
        self.key_color = Some(key_color);
        self
    }
}

/// Tileset holds a collection of tiles stored as their pixel data.
///
/// Currently only supports RGBA 8 bits per channel.
///
/// Tiles are counted left-to-right then top-to-bottom.
///
/// Generic parameter `C` is the container type, which should implement `AsRef<[u8]>`.
/// You can use a simple `Vec<u8>`/`&[u8]` with RGBA data, `Rc<[u8]>`/`Arc<[u8]>` for cheap cloning
/// or e.g. `image`'s [`ImageBuffer`](https://docs.rs/image/latest/image/struct.ImageBuffer.html).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tileset<C> {
    data: C,
    width: u32,
    height: u32,
    tile_counts: (u32, u32),
    pub(crate) opts: TilesetOptions,
}

impl<C> Tileset<C> {
    /// Whether `id` is a valid tile id for this tileset.
    ///
    /// Basically `id < self.tile_count()`.
    #[inline]
    pub fn contains(&self, id: TileId) -> bool {
        id < self.tile_count()
    }

    /// Total amount of tiles in the tileset.
    #[inline]
    pub fn tile_count(&self) -> u32 {
        self.tile_counts.0 * self.tile_counts.1
    }

    /// Tileset options used when creating the tileset.
    #[inline]
    pub fn options(&self) -> &TilesetOptions {
        &self.opts
    }
}

impl<C> Tileset<C>
where
    C: AsRef<[u8]>,
{
    /// Construct a new tileset.
    /// `width` and `height` are `data`'s size in pixels.
    pub fn new(data: C, width: u32, height: u32, opts: TilesetOptions) -> Option<Self> {
        if data.as_ref().len() == ((width * height) as usize * size_of::<C>()) {
            let tile_counts = calc_tile_counts(width, height, &opts);

            Some(Self {
                data,
                width,
                height,
                tile_counts,
                opts,
            })
        } else {
            None
        }
    }

    /// Get the position of a tile in the tileset.
    /// Useful if you need to render a single tile.
    pub fn get_tile_pos(&self, id: TileId) -> Option<(u32, u32)> {
        let x = (id % self.tile_counts.0) * (self.opts.tile_size.0 + self.opts.margin.0)
            + self.opts.offset.0;

        let y = (id / self.tile_counts.0) * (self.opts.tile_size.1 + self.opts.margin.1)
            + self.opts.offset.1;

        if (x + self.opts.tile_size.0) < self.width as _
            && (y + self.opts.tile_size.1) < self.height as _
        {
            Some((x, y))
        } else {
            None
        }
    }
}

impl<C> Buffer<Color> for Tileset<C>
where
    C: AsRef<[u8]>,
{
    #[inline]
    fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    fn get(&self, x: u32, y: u32) -> &Color {
        self.data
            .as_ref()
            .as_pixels()
            .index((y * self.width + x) as usize)
    }
}

impl<C> BufferMut<Color> for Tileset<C>
where
    C: AsRef<[u8]> + AsMut<[u8]>,
{
    #[inline]
    fn get_mut(&mut self, x: u32, y: u32) -> &mut Color {
        self.data
            .as_mut()
            .as_pixels_mut()
            .index_mut((y * self.width + x) as usize)
    }
}

#[inline]
const fn calc_tile_counts(width: u32, height: u32, opts: &TilesetOptions) -> (u32, u32) {
    (
        (width - opts.offset.0 + opts.margin.0) / (opts.tile_size.0 + opts.margin.0),
        (height - opts.offset.1 + opts.margin.1) / (opts.tile_size.1 + opts.margin.1),
    )
}
