///! Provides utilities for 2D Morton code generation using Fabian Giesen's Morton
///! code decoding functions, see [his post on Morton codes](https://fgiesen.wordpress.com/2009/12/13/decoding-morton-codes/)

/// Insert a 0 bit between each of the low 16 bits of x
pub fn part1_by1(mut x: u32) -> u32 {
	// x = ---- ---- ---- ---- fedc ba98 7654 3210
	x &= 0x0000ffff;
	// x = ---- ---- fedc ba98 ---- ---- 7654 3210
	x = (x ^ (x << 8)) & 0x00ff00ff;
	// x = ---- fedc ---- ba98 ---- 7654 ---- 3210
	x = (x ^ (x << 4)) & 0x0f0f0f0f;
	// x = --fe --dc --ba --98 --76 --54 --32 --10
	x = (x ^ (x << 2)) & 0x33333333;
	// x = -f-e -d-c -b-a -9-8 -7-6 -5-4 -3-2 -1-0
	(x ^ (x << 1)) & 0x55555555
}
/// Compute the Morton code for the `(x, y)` position
pub fn morton2(p: &(u32, u32)) -> u32 {
	(part1_by1(p.1) << 1) + part1_by1(p.0)
}

