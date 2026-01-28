#define PLATFORM_WEBGL
#line 0
#ifdef GL_ES
precision mediump float;
#endif 
#define PI 3.1415926535 
#define HALF_PI 1.57079632679 

uniform vec2 u_resolution;
uniform float u_time;

float dither8x8(vec2 position, float brightness) {
  int x = int(mod(position.x, 8.0));
  int y = int(mod(position.y, 8.0));
  int index = x + y * 8;
  float limit = 0.0;

  if (x < 8) {
    if (index == 0)
        limit = 0.015625;
    if (index == 1)
        limit = 0.515625;
    if (index == 2)
        limit = 0.140625;
    if (index == 3)
        limit = 0.640625;
    if (index == 4)
        limit = 0.046875;
    if (index == 5)
        limit = 0.546875;
    if (index == 6)
        limit = 0.171875;
    if (index == 7)
        limit = 0.671875;
    if (index == 8)
        limit = 0.765625;
    if (index == 9)
        limit = 0.265625;
    if (index == 10)
        limit = 0.890625;
    if (index == 11)
        limit = 0.390625;
    if (index == 12)
        limit = 0.796875;
    if (index == 13)
        limit = 0.296875;
    if (index == 14)
        limit = 0.921875;
    if (index == 15)
        limit = 0.421875;
    if (index == 16)
        limit = 0.203125;
    if (index == 17)
        limit = 0.703125;
    if (index == 18)
        limit = 0.078125;
    if (index == 19)
        limit = 0.578125;
    if (index == 20)
        limit = 0.234375;
    if (index == 21)
        limit = 0.734375;
    if (index == 22)
        limit = 0.109375;
    if (index == 23)
        limit = 0.609375;
    if (index == 24)
        limit = 0.953125;
    if (index == 25)
        limit = 0.453125;
    if (index == 26)
        limit = 0.828125;
    if (index == 27)
        limit = 0.328125;
    if (index == 28)
        limit = 0.984375;
    if (index == 29)
        limit = 0.484375;
    if (index == 30)
        limit = 0.859375;
    if (index == 31)
        limit = 0.359375;
    if (index == 32)
        limit = 0.0625;
    if (index == 33)
        limit = 0.5625;
    if (index == 34)
        limit = 0.1875;
    if (index == 35)
        limit = 0.6875;
    if (index == 36)
        limit = 0.03125;
    if (index == 37)
        limit = 0.53125;
    if (index == 38)
        limit = 0.15625;
    if (index == 39)
        limit = 0.65625;
    if (index == 40)
        limit = 0.8125;
    if (index == 41)
        limit = 0.3125;
    if (index == 42)
        limit = 0.9375;
    if (index == 43)
        limit = 0.4375;
    if (index == 44)
        limit = 0.78125;
    if (index == 45)
        limit = 0.28125;
    if (index == 46)
        limit = 0.90625;
    if (index == 47)
        limit = 0.40625;
    if (index == 48)
        limit = 0.25;
    if (index == 49)
        limit = 0.75;
    if (index == 50)
        limit = 0.125;
    if (index == 51)
        limit = 0.625;
    if (index == 52)
        limit = 0.21875;
    if (index == 53)
        limit = 0.71875;
    if (index == 54)
        limit = 0.09375;
    if (index == 55)
        limit = 0.59375;
    if (index == 56)
        limit = 1.0;
    if (index == 57)
        limit = 0.5;
    if (index == 58)
        limit = 0.875;
    if (index == 59)
        limit = 0.375;
    if (index == 60)
        limit = 0.96875;
    if (index == 61)
        limit = 0.46875;
    if (index == 62)
        limit = 0.84375;
    if (index == 63)
        limit = 0.34375;
  }

  return brightness < limit ? 0.0 : 1.0;
}

// Noise
vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
vec4 mod289(vec4 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
vec4 permute(vec4 x) { return mod289(((x*34.0)+1.0)*x); }
vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }
vec3 fade(vec3 t) { return t*t*t*(t*(t*6.0-15.0)+10.0); }
float noise(vec3 P) {
    vec3 i0 = mod289(floor(P)), i1 = mod289(i0 + vec3(1.0));
    vec3 f0 = fract(P), f1 = f0 - vec3(1.0), f = fade(f0);
    vec4 ix = vec4(i0.x, i1.x, i0.x, i1.x), iy = vec4(i0.yy, i1.yy);
    vec4 iz0 = i0.zzzz, iz1 = i1.zzzz;
    vec4 ixy = permute(permute(ix) + iy), ixy0 = permute(ixy + iz0), ixy1 = permute(ixy + iz1);
    vec4 gx0 = ixy0 * (1.0 / 7.0), gy0 = fract(floor(gx0) * (1.0 / 7.0)) - 0.5;
    vec4 gx1 = ixy1 * (1.0 / 7.0), gy1 = fract(floor(gx1) * (1.0 / 7.0)) - 0.5;
    gx0 = fract(gx0); gx1 = fract(gx1);
    vec4 gz0 = vec4(0.5) - abs(gx0) - abs(gy0), sz0 = step(gz0, vec4(0.0));
    vec4 gz1 = vec4(0.5) - abs(gx1) - abs(gy1), sz1 = step(gz1, vec4(0.0));
    gx0 -= sz0 * (step(0.0, gx0) - 0.5); gy0 -= sz0 * (step(0.0, gy0) - 0.5);
    gx1 -= sz1 * (step(0.0, gx1) - 0.5); gy1 -= sz1 * (step(0.0, gy1) - 0.5);
    vec3 g0 = vec3(gx0.x,gy0.x,gz0.x), g1 = vec3(gx0.y,gy0.y,gz0.y),
        g2 = vec3(gx0.z,gy0.z,gz0.z), g3 = vec3(gx0.w,gy0.w,gz0.w),
        g4 = vec3(gx1.x,gy1.x,gz1.x), g5 = vec3(gx1.y,gy1.y,gz1.y),
        g6 = vec3(gx1.z,gy1.z,gz1.z), g7 = vec3(gx1.w,gy1.w,gz1.w);
    vec4 norm0 = taylorInvSqrt(vec4(dot(g0,g0), dot(g2,g2), dot(g1,g1), dot(g3,g3)));
    vec4 norm1 = taylorInvSqrt(vec4(dot(g4,g4), dot(g6,g6), dot(g5,g5), dot(g7,g7)));
    g0 *= norm0.x; g2 *= norm0.y; g1 *= norm0.z; g3 *= norm0.w;
    g4 *= norm1.x; g6 *= norm1.y; g5 *= norm1.z; g7 *= norm1.w;
    vec4 nz = mix(vec4(dot(g0, vec3(f0.x, f0.y, f0.z)), dot(g1, vec3(f1.x, f0.y, f0.z)),
        dot(g2, vec3(f0.x, f1.y, f0.z)), dot(g3, vec3(f1.x, f1.y, f0.z))),
        vec4(dot(g4, vec3(f0.x, f0.y, f1.z)), dot(g5, vec3(f1.x, f0.y, f1.z)),
            dot(g6, vec3(f0.x, f1.y, f1.z)), dot(g7, vec3(f1.x, f1.y, f1.z))), f.z);
    return 2.2 * mix(mix(nz.x,nz.z,f.y), mix(nz.y,nz.w,f.y), f.x);
}

float dither_noise(vec2 position, float brightness) {
    return noise(vec3(position, 0.0) * .7) + 1. > brightness ? 0.0 : 1.0;
}

float turbulence(vec3 P) {
    float f = 0., s = 1.;
    for (int i = 0 ; i < 5 ; i++) {
        f += abs(noise(s * P)) / s;
        s *= 2.;
        P = vec3(.866 * P.x + .5 * P.z, P.y + 100., -.5 * P.x + .866 * P.z);
    }
    return f;
}

void main(){
    vec2 st = gl_FragCoord.xy/u_resolution.xy;
    float L = turbulence(vec3(st, u_time * .015));
    float brightness = noise(vec3(.5, .5, L) * .7) + 1.0;
    float db = dither8x8(gl_FragCoord.xy, brightness);
    vec3 color = bool(db > 0.0) ? vec3(0.937255, 0.960784, 0.937255) : vec3(0.137255, 0.141176, 0.141176);
    gl_FragColor = vec4(color, 1.);
}

