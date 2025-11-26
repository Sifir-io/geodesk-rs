# Build Fix for GeoDESK Upstream Bug

## Issue
The upstream libgeodesk library v2.1.0 (commit `0e4baedf470496f90440a99e2a1cdaf4a17f04eb`) contains a syntax error in `src/feature/FeatureIteratorBase.cpp` at line 142:

```cpp
storage_.nodes.cursor.~WayNodeCursor()();  // INCORRECT - extra ()
```

This causes a compilation error:
```
error: void value not ignored as it ought to be
```

## Root Cause
The destructor call has an extra set of parentheses, treating the destructor call as a function that returns void and then attempting to call that void value.

## Solution
The CMakeLists.txt has been updated to automatically patch the upstream source after fetching it from GitHub. The fix uses `sed` to correct the destructor syntax:

```cmake
execute_process(
    COMMAND sed -i "s/\\.~WayNodeCursor()()/\\.~WayNodeCursor()/g"
            "${geodesk_SOURCE_DIR}/src/feature/FeatureIteratorBase.cpp"
    RESULT_VARIABLE PATCH_RESULT
)
```

This changes line 142 to:
```cpp
storage_.nodes.cursor.~WayNodeCursor();  // CORRECT
```

## Status
- ✅ Build now succeeds
- ✅ Patch is automatically applied during CMake FetchContent
- ✅ Pinned to specific commit: `0e4baedf470496f90440a99e2a1cdaf4a17f04eb` (v2.1.0)
- ⚠️ This is a workaround for an upstream bug

## Future Considerations
1. Monitor the upstream repository for a proper fix
2. When upstream fixes the bug, update to that commit and remove the patch
3. Consider reporting the issue to the upstream maintainers if not already done
