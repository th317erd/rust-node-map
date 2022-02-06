use super::Index;

fn crunch_lerp(value: f64, min: f64, max: f64, range: u32) -> Index {
  let lerp_range: f64 = max - min;
  (((value - min) / lerp_range) * range as f64) as Index
}

pub trait NodeMapIndexSize<T> {
  fn node_map_index_size() -> usize;
  fn node_map_crunch(&self, min: T, max: T) -> Index;
  fn node_map_smallest_value() -> T;
  fn node_map_largest_value() -> T;
}

macro_rules! node_map_index {
  ($size:expr; $($type:ty),+ $(,)?) => {
    $(
      impl NodeMapIndexSize<$type> for $type {
        fn node_map_index_size() -> usize {
          return $size + 1;
        }

        fn node_map_smallest_value() -> $type {
          return <$type>::MIN;
        }

        fn node_map_largest_value() -> $type {
          return <$type>::MAX;
        }

        fn node_map_crunch(&self, min: $type, max: $type) -> Index {
          return crunch_lerp(*self as f64, min as f64, max as f64, $size)
        }
      }
    )+
  };
}

impl NodeMapIndexSize<bool> for bool {
  fn node_map_index_size() -> usize {
    return 2;
  }

  fn node_map_smallest_value() -> bool {
    return false;
  }

  fn node_map_largest_value() -> bool {
    return true;
  }

  fn node_map_crunch(&self, _min: bool, _max: bool) -> Index {
    return *self as u32;
  }
}

node_map_index![255; i8, u8];
node_map_index![65535; i16, u16, i32, u32];
node_map_index![1048575; i64, u64, f32];

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn has_correct_index_size_for_types() {
    assert_eq!(bool::node_map_index_size(), 2);
    assert_eq!(i8::node_map_index_size(), 256);
    assert_eq!(u8::node_map_index_size(), 256);
    assert_eq!(i16::node_map_index_size(), 65536);
    assert_eq!(u16::node_map_index_size(), 65536);
    assert_eq!(i32::node_map_index_size(), 65536);
    assert_eq!(u32::node_map_index_size(), 65536);
    assert_eq!(i64::node_map_index_size(), 1048576);
    assert_eq!(u64::node_map_index_size(), 1048576);
    assert_eq!(f32::node_map_index_size(), 1048576);
  }

  #[test]
  fn crunches_correctly() {
    // bool
    assert_eq!(false.node_map_crunch(false, true), 0);
    assert_eq!(true.node_map_crunch(false, true), 1);

    // i8
    assert_eq!(i8::MIN.node_map_crunch(i8::MIN, i8::MAX), 0);
    assert_eq!(0i8.node_map_crunch(i8::MIN, i8::MAX), 128);
    assert_eq!(i8::MAX.node_map_crunch(i8::MIN, i8::MAX), 255);

    assert_eq!((-27i8).node_map_crunch(-27, 127), 0);
    assert_eq!(0i8.node_map_crunch(-27, 127), 44);
    assert_eq!(127i8.node_map_crunch(-27, 127), 255);

    // // u8
    assert_eq!(u8::MIN.node_map_crunch(u8::MIN, u8::MAX), 0);
    assert_eq!(u8::MAX.node_map_crunch(u8::MIN, u8::MAX), 255);

    // // i16
    assert_eq!(i16::MIN.node_map_crunch(i16::MIN, i16::MAX), 0);
    assert_eq!(0i16.node_map_crunch(i16::MIN, i16::MAX), 32768);
    assert_eq!(i16::MAX.node_map_crunch(i16::MIN, i16::MAX), 65535);

    // // u16
    assert_eq!(u16::MIN.node_map_crunch(u16::MIN, u16::MAX), 0);
    assert_eq!(u16::MAX.node_map_crunch(u16::MIN, u16::MAX), 65535);

    // // i32
    assert_eq!(i32::MIN.node_map_crunch(i32::MIN, i32::MAX), 0);
    assert_eq!(0i32.node_map_crunch(i32::MIN, i32::MAX), 32767);
    assert_eq!(i32::MAX.node_map_crunch(i32::MIN, i32::MAX), 65535);

    // // u32
    assert_eq!(u32::MIN.node_map_crunch(u32::MIN, u32::MAX), 0);
    assert_eq!(u32::MAX.node_map_crunch(u32::MIN, u32::MAX), 65535);

    // // i64
    assert_eq!(i64::MIN.node_map_crunch(i64::MIN, i64::MAX), 0);
    assert_eq!(0i64.node_map_crunch(i64::MIN, i64::MAX), 524287);
    assert_eq!(i64::MAX.node_map_crunch(i64::MIN, i64::MAX), 1048575);

    // // u64
    assert_eq!(u64::MIN.node_map_crunch(u64::MIN, u64::MAX), 0);
    assert_eq!(u64::MAX.node_map_crunch(u64::MIN, u64::MAX), 1048575);

    // // f32
    assert_eq!(f32::MIN.node_map_crunch(f32::MIN, f32::MAX), 0);
    assert_eq!(0f32.node_map_crunch(f32::MIN, f32::MAX), 524287);
    assert_eq!(f32::MAX.node_map_crunch(f32::MIN, f32::MAX), 1048575);

    assert_eq!(0f32.node_map_crunch(-10f32, 10f32), 524287);
  }
}
