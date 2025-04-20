# Reflectance Imaging (RefIm)
## Expected Result
### Case 1
```
[255 255 255] - [255   0   0] = [  0 255 255] == [255   0   0]
[255 255 255] - [255 255 255] = [  0   0   0] == [  0   0   0]
```

### Case 2
```
[  0   0   0] - [255   0   0] = [255   0   0] == [255   0   0]
[  0   0   0] - [  0   0   0] = [  0   0   0] == [  0   0   0]
```

## Caveats / Limitations
1. Colored light on an object with the same color is considered
   invisible.
