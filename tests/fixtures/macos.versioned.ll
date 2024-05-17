; ModuleID = 'Unnamed_module'
source_filename = "Unnamed_module"

@GTab = global [1 x i32] [i32 0]
@GMem = global <{ i64, i64 }> <{ i64 17, i64 0 }>, section ",_memory", align 1
@GVar = internal global i32 1048576
@GVar.1 = internal global i32 1082429
@GVar.2 = internal global i32 1082432
@GFnPar = internal global [11 x i64] zeroinitializer
@GInitMem = global <{ i32, i64, i64, [1 x i8] }> <{ i32 0, i64 1048576, i64 33368, [1 x i8] c"c" }>, section ",_init_memory", align 1

; Function Attrs: nofree nosync nounwind
define dso_local void @ft_transfer() #0 {
Entry:
	ret void
}

attributes #0 = { nofree nosync nounwind "frame-pointer"="all" "no-trapping-math"="true" }
@_OBJECT_VERSION = global i64 1, section "_version", align 1
@_EXPECTED_RUNTIME_VERSION = global i64 3, section "_version", align 1
