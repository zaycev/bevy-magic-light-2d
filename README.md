# 🔮 Bevy Magic Light 2D – experimental dynamic 2D global illumination system.

> 🚧 Beware! This repo is heavily under construction and a lot of things may change,

Experimental dynamic 2D global illumination system for Bevy, based on SDF raymarching and screen space irradiance cache probes.

![alt text](https://github.com/zaycev/bevy-2d-gi-experiment/blob/main/static/gif.gif?raw=true "Title")

### Lighting and Shadows

This part is calculating amount of light received by surface for each pixel on a screen.

#### Direct illumination

1. Calculate SDF map containing min distance from each occluder on a screen.

![alt text](https://github.com/zaycev/bevy-2d-gi-experiment/blob/main/static/pixl_sdf.png?raw=true "Title")

2. Use calculated SDF map to check if sampled point is occluded or not using Ray Marching.
   1. In this step we can calculcate heuristic for panumbras. To do that we can remember the distance from a ray to the closest occluder and lower light contribution for the rays that are closer to the occluders.

![alt text](https://github.com/zaycev/bevy-2d-gi-experiment/blob/main/static/pixel_attenuation.png?raw=true "Title")

4. Calculate attenuation using one of the methods. A form of quadratic falloff is used in this experiment.

***TODO***

#### Indirect illumination

1. ***TODO***
2.
3.

### Shading

This part is calculating amount of light reflected from surface toward the camera.

***TODO***

### References

- [Ray Marching Soft Shadows in 2D](https://www.rykap.com/2020/09/23/distance-fields/)
- [Soft Shadows in Raymarched SDFs](https://iquilezles.org/articles/rmshadows/)
- [Free blue noise textures](http://momentsingraphics.de/BlueNoise.html)
