#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::slice::from_raw_parts_mut;

use console::*;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, OriginDimensions, PixelColor, Point, Primitive, RgbColor, Size},
    primitives::{PrimitiveStyle, Rectangle},
    Drawable, Pixel,
};
use oorandom::Rand32;
use user_lib::*;

const VIRTGPU_XRES: usize = 1280;
const VIRTGPU_YRES: usize = 800;
const VIRTGPU_LEN: usize = VIRTGPU_XRES * VIRTGPU_YRES;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;

pub struct Display {
    pub size: Size,
    pub point: Point,
    pub fb: &'static mut [u8],
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    None,
}

pub struct Snake<T: PixelColor, const MAX_SIZE: usize> {
    parts: [Pixel<T>; MAX_SIZE],
    len: usize,
    direction: Direction,
    size_x: u32,
    size_y: u32,
}

pub struct SnakeIntoIterator<'a, T: PixelColor, const MAX_SIZE: usize> {
    snake: &'a Snake<T, MAX_SIZE>,
    index: usize,
}

impl<'a, T: PixelColor, const MAX_SIZE: usize> Iterator for SnakeIntoIterator<'a, T, MAX_SIZE> {
    type Item = Pixel<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.snake.parts[self.index];
        if self.index < self.snake.len {
            self.index += 1;
            return Some(cur);
        } else {
            return None;
        }
    }
}

impl<'a, T: PixelColor, const MAX_SIZE: usize> IntoIterator for &'a Snake<T, MAX_SIZE> {
    type Item = Pixel<T>;

    type IntoIter = SnakeIntoIterator<'a, T, MAX_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        SnakeIntoIterator {
            snake: self,
            index: 0,
        }
    }
}

pub struct Food<T: PixelColor> {
    size_x: u32,
    size_y: u32,
    place: Pixel<T>,
    rng: Rand32,
}

pub struct SnakeGame<const MAX_SNAKE_SIZE: usize, T: PixelColor> {
    snake: Snake<T, MAX_SNAKE_SIZE>,
    food: Food<T>,
    food_age: u32,
    food_lifetime: u32,
    size_x: u32,
    size_y: u32,
    scale_x: u32,
    scale_y: u32,
}

struct ScaledDisplay<'a, T: DrawTarget> {
    real_display: &'a mut T,
    size_x: u32,
    size_y: u32,
    scale_x: u32,
    scale_y: u32,
}

impl<'a, T: DrawTarget> DrawTarget for ScaledDisplay<'a, T> {
    type Color = T::Color;

    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let style = PrimitiveStyle::with_fill(pixel.1);
            Rectangle::new(
                Point::new(
                    pixel.0.x * self.scale_x as i32,
                    pixel.0.y * self.scale_y as i32,
                ),
                Size::new(self.scale_x as u32, self.scale_y as u32),
            )
            .into_styled(style)
            .draw(self.real_display)?;
        }
        Ok(())
    }
}

impl<'a, T: DrawTarget> OriginDimensions for ScaledDisplay<'a, T> {
    fn size(&self) -> Size {
        Size::new(self.size_x as u32, self.size_y as u32)
    }
}

impl<const MAX_SIZE: usize, T: PixelColor> SnakeGame<MAX_SIZE, T> {
    pub fn new(
        size_x: u32,
        size_y: u32,
        scale_x: u32,
        scale_y: u32,
        sanke_color: T,
        food_color: T,
        food_lifetime: u32,
    ) -> Self {
        let snake = Snake::<T, MAX_SIZE>::new(sanke_color, size_x / scale_x, size_y / scale_y);
        let mut food = Food::<T>::new(food_color, size_x, size_y);
        food.replace(&snake);
        SnakeGame {
            snake: snake,
            food,
            food_age: 0,
            food_lifetime,
            size_x,
            size_y,
            scale_x,
            scale_y,
        }
    }
    pub fn set_direction(&mut self, direction: Direction) {
        self.snake.set_direction(direction);
    }
    pub fn draw<D>(&mut self, target: &mut D) -> ()
    where
        D: DrawTarget<Color = T>,
    {
        self.snake.make_step();
        let hit = self.snake.contains(self.food.get_pixel().0);
        if hit {
            self.snake.grow();
        }
        self.food_age += 1;
        if self.food_age > self.food_lifetime || hit {
            self.food.replace(&self.snake);
            self.food_age = 0;
        }

        let mut scaled_display = ScaledDisplay::<D> {
            real_display: target,
            size_x: self.size_x / self.scale_x,
            size_y: self.size_y / self.scale_y,
            scale_x: self.scale_x,
            scale_y: self.scale_y,
        };

        for part in self.snake.into_iter() {
            _ = part.draw(&mut scaled_display);
        }

        _ = self.food.get_pixel().draw(&mut scaled_display);
    }
}

impl<T: PixelColor> Food<T> {
    pub fn new(color: T, size_x: u32, size_y: u32) -> Self {
        Self {
            size_x,
            size_y,
            place: Pixel(Point { x: 0, y: 0 }, color),
            rng: Rand32::new(32),
        }
    }

    fn replace<'a, const MAX_SIZE: usize>(&mut self, iter_source: &Snake<T, MAX_SIZE>) {
        let mut p: Point;
        'outer: loop {
            let randdom_number = self.rng.rand_u32();
            let blocked_positions = iter_source.into_iter();
            p = Point {
                x: ((randdom_number >> 24) as u16 % self.size_x as u16).into(),
                y: ((randdom_number >> 16) as u16 % self.size_y as u16).into(),
            };
            for blocked_positio in blocked_positions {
                if p == blocked_positio.0 {
                    continue 'outer;
                }
            }
            break;
        }

        self.place = Pixel::<T>(p, self.place.1);
    }
    fn get_pixel(&self) -> Pixel<T> {
        self.place
    }
}

impl<T: PixelColor, const MAX_SIZE: usize> Snake<T, MAX_SIZE> {
    pub fn new(color: T, size_x: u32, size_y: u32) -> Self {
        Snake {
            parts: [Pixel::<T>(Point { x: 0, y: 0 }, color); MAX_SIZE],
            len: 1,
            direction: Direction::None,
            size_x,
            size_y,
        }
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn make_step(&mut self) {
        let mut i = self.len;
        while i > 0 {
            self.parts[i] = self.parts[i - 1];
            i -= 1;
        }
        match self.direction {
            Direction::Left => {
                if self.parts[0].0.x == 0 {
                    self.parts[0].0.x = (self.size_x - 1) as i32;
                } else {
                    self.parts[0].0.x -= 1;
                }
            }
            Direction::Right => {
                if self.parts[0].0.x == (self.size_x - 1) as i32 {
                    self.parts[0].0.x = 0;
                } else {
                    self.parts[0].0.x += 1;
                }
            }
            Direction::Up => {
                if self.parts[0].0.y == 0 {
                    self.parts[0].0.y = (self.size_y - 1) as i32;
                } else {
                    self.parts[0].0.y -= 1;
                }
            }
            Direction::Down => {
                if self.parts[0].0.y == (self.size_y - 1) as i32 {
                    self.parts[0].0.y = 0;
                } else {
                    self.parts[0].0.y += 1;
                }
            }
            Direction::None => {}
        }
    }

    fn contains(&self, this: Point) -> bool {
        self.into_iter().find(|p| p.0 == this).is_some()
    }

    fn grow(&mut self) {
        if self.len < MAX_SIZE - 1 {
            self.len += 1;
        }
    }
}

impl Display {
    pub fn new(size: Size, point: Point) -> Self {
        let fb_ptr = framebuffer() as *mut u8;
        println!(
            "Hello world from user mode program! 0x{:X} , len {}",
            fb_ptr as usize, VIRTGPU_LEN as usize
        );
        let fb = unsafe { from_raw_parts_mut(fb_ptr as *mut u8, VIRTGPU_LEN as usize) };
        Self { size, point, fb }
    }

    pub fn flush(&self) {
        framebuffer_flush();
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        pixels.into_iter().for_each(|px| {
            let idx = (px.0.y * VIRTGPU_XRES as i32 + px.0.x) as usize * 4;
            if idx + 2 >= self.fb.len() {
                return;
            }
            self.fb[idx] = px.1.b();
            self.fb[idx + 1] = px.1.g();
            self.fb[idx + 2] = px.1.r();
        });
        Ok(())
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        self.size
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::pixelcolor::*;
    use embedded_graphics::prelude::*;

    use crate::Snake;

    #[test]
    fn snake_basic() {
        let mut snake = Snake::<Rgb888, 20>::new(Rgb888::RED, 8, 8);
        snake.set_direction(crate::Direction::Right);
        assert_eq!(
            Pixel::<Rgb888>(Point { x: 0, y: 0 }, Rgb888::RED),
            snake.into_iter().next().unwrap()
        );
        snake.make_step();
        assert_eq!(
            Pixel::<Rgb888>(Point { x: 1, y: 0 }, Rgb888::RED),
            snake.into_iter().nth(0).unwrap()
        );
        assert_eq!(
            Pixel::<Rgb888>(Point { x: 0, y: 0 }, Rgb888::RED),
            snake.into_iter().nth(1).unwrap()
        );
        snake.set_direction(crate::Direction::Down);
        snake.make_step();
        assert_eq!(
            Pixel::<Rgb888>(Point { x: 1, y: 1 }, Rgb888::RED),
            snake.into_iter().nth(0).unwrap()
        );
        assert_eq!(
            Pixel::<Rgb888>(Point { x: 1, y: 0 }, Rgb888::RED),
            snake.into_iter().nth(1).unwrap()
        );
        assert_eq!(
            Pixel::<Rgb888>(Point { x: 0, y: 0 }, Rgb888::RED),
            snake.into_iter().nth(2).unwrap()
        );
        assert_eq!(true, snake.contains(Point { x: 0, y: 0 }));
        assert_eq!(true, snake.contains(Point { x: 1, y: 0 }));
        assert_eq!(true, snake.contains(Point { x: 1, y: 1 }));
    }
}

#[no_mangle]
pub fn main() -> i32 {
    let mut disp = Display::new(
        Size::new(VIRTGPU_XRES as u32, VIRTGPU_YRES as u32),
        Point::new(0, 0),
    );
    let mut game = SnakeGame::<20, Rgb888>::new(
        VIRTGPU_XRES as u32,
        VIRTGPU_YRES as u32,
        20,
        20,
        Rgb888::RED,
        Rgb888::YELLOW,
        200,
    );
    disp.clear(Rgb888::BLACK).unwrap();
    loop {
        if key_pressed() {
            let c = getchar();
            match c {
                LF => break,
                CR => break,
                b'w' => game.set_direction(Direction::Up),
                b's' => game.set_direction(Direction::Down),
                b'a' => game.set_direction(Direction::Left),
                b'd' => game.set_direction(Direction::Right),
                _ => (),
            }
        }
        let _ = disp.clear(Rgb888::BLACK).unwrap();
        game.draw(&mut disp);
        disp.flush();
        sleep(40);
    }
    0
}
