using System.Runtime.InteropServices;
using System.Runtime.CompilerServices;
using Sdk = Lumio.Engine.SDK;
using Xunit;

namespace Lumio.Engine.NativeLoader.Tests;

public sealed class VoxelFacadeTests
{
    private static readonly Sdk.VoxelReadCellFn ReadCell = ReadCellImpl;
    private static readonly Sdk.VoxelReadBatchFn ReadBatch = ReadBatchImpl;
    private static readonly Sdk.VoxelPrepareFn Prepare = PrepareImpl;
    private static readonly Sdk.VoxelPrepareFn PrepareWaitsForLeaseDispose = PrepareWaitsForLeaseDisposeImpl;
    private static readonly Sdk.VoxelCommitFn Commit = CommitImpl;
    private static readonly Sdk.VoxelAbortFn Abort = AbortImpl;
    private static readonly Sdk.VoxelRevisionFn Revision = RevisionImpl;
    private static readonly Sdk.VoxelRaycastFn Raycast = RaycastImpl;
    private static readonly Sdk.VoxelSweepFn Sweep = SweepImpl;
    private static readonly Sdk.VoxelOverlapFn Overlap = OverlapImpl;
    private static int AbortCalls;
    private static ManualResetEventSlim? PrepareEntered;
    private static ManualResetEventSlim? LeaseDisposed;

    [Fact]
    public void CellReadPreservesPendingPresenceWithoutInventingAir()
    {
        var api = CreateApi(
            blockReadCell: Pointer(ReadCell));
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);

        var result = world.ReadCell(new Sdk.VoxelWorldCoordinate(4, 255, -7));

        Assert.Equal(Sdk.VoxelPresence.Pending, result.Presence);
        Assert.False(result.HasBlockId);
        Assert.Equal(0u, result.BlockId);
        Assert.Equal(17ul, result.SectionRevision);
    }

    [Fact]
    public void BoxReadWritesCallerBuffersAndReturnsNativeCounts()
    {
        var api = CreateApi(blockReadBox: Pointer(ReadBatch));
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);
        var cells = new Sdk.VoxelBlockReadResult[2];
        var segments = new Sdk.VoxelSectionSegment[1];

        var outcome = world.ReadBox(
            new Sdk.VoxelBoxRequest(new Sdk.VoxelWorldCoordinate(0, 1, 0), new Sdk.VoxelWorldCoordinate(1, 1, 0)),
            cells,
            segments);

        Assert.Equal(2u, outcome.ResultCount);
        Assert.Equal(1u, outcome.SegmentCount);
        Assert.True(outcome.Truncated);
        Assert.Equal(Sdk.VoxelPresence.Ready, cells[0].Presence);
        Assert.Equal(0x80000123u, cells[0].BlockId);
        Assert.Equal(22ul, cells[0].SectionRevision);
        Assert.Equal(2u, segments[0].ResultCount);
    }

    [Fact]
    public void WritePrepareCommitUsesOpaqueTokenAndCallerReceipts()
    {
        var api = CreateApi(
            blockWritePrepare: Pointer(Prepare),
            blockWriteCommit: Pointer(Commit),
            blockWriteAbort: Pointer(Abort));
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);
        var token = world.PrepareWrite(
            42,
            new[] { new Sdk.VoxelBlockWriteEntry(new Sdk.VoxelSectionKey(1, 2, 3), 4095, uint.MaxValue, 9) });
        var receipts = new Sdk.VoxelWriteReceipt[1];

        var count = world.Commit(token, receipts);

        Assert.Equal(1u, count);
        Assert.Equal(1, receipts[0].SectionKey.X);
        Assert.Equal(10ul, receipts[0].UpToSectionRevision);
        Assert.Equal(99ul, receipts[0].WorldRevision);
        Assert.True(token.IsCompleted);
    }

    [Fact]
    public void LateTokenDisposeAbortsAfterLeaseDisposeAndReleasesLeaseReference()
    {
        AbortCalls = 0;
        var lease = CreateLease(CreateApi(
            blockWritePrepare: Pointer(Prepare),
            blockWriteAbort: Pointer(Abort)));
        var world = lease.CreateVoxelWorld((nint)0x1240);
        var token = world.PrepareWrite(43, Array.Empty<Sdk.VoxelBlockWriteEntry>());

        lease.Dispose();

        token.Dispose();
        token.Dispose();

        Assert.True(token.IsCompleted);
        Assert.Equal(1, AbortCalls);
        Assert.Equal(0, lease.ActiveNativeTokenCount);
    }

    [Fact]
    public async Task PrepareRetainsLeaseBeforeConcurrentDisposeCanUnloadNativeLibrary()
    {
        AbortCalls = 0;
        using var prepareEntered = new ManualResetEventSlim(false);
        using var leaseDisposed = new ManualResetEventSlim(false);
        var cancellationToken = TestContext.Current.CancellationToken;
        PrepareEntered = prepareEntered;
        LeaseDisposed = leaseDisposed;
        NativeEngineLease? lease = null;
        try
        {
            lease = CreateLease(CreateApi(
                blockWritePrepare: Pointer(PrepareWaitsForLeaseDispose),
                blockWriteAbort: Pointer(Abort)));
            var world = lease.CreateVoxelWorld((nint)0x1244);
            var prepareTask = Task.Run(
                () => world.PrepareWrite(46, Array.Empty<Sdk.VoxelBlockWriteEntry>()),
                cancellationToken);

            Assert.True(prepareEntered.Wait(TimeSpan.FromSeconds(5), cancellationToken));
            var disposeTask = Task.Run(
                () =>
                {
                    lease.Dispose();
                    leaseDisposed.Set();
                },
                cancellationToken);
            Assert.True(leaseDisposed.Wait(TimeSpan.FromSeconds(5), cancellationToken));
            await disposeTask;

            using var token = await prepareTask;
            Assert.False(token.IsCompleted);
            Assert.Equal(1, lease.ActiveNativeTokenCount);
            token.Dispose();
            Assert.Equal(1, AbortCalls);
            Assert.Equal(0, lease.ActiveNativeTokenCount);
            Assert.True(disposeTask.IsCompletedSuccessfully);
        }
        finally
        {
            PrepareEntered = null;
            LeaseDisposed = null;
            lease?.Dispose();
        }
    }

    [Fact]
    public void ForgottenTokenIsAbortedBySafeHandleFinalization()
    {
        AbortCalls = 0;
        var weakToken = CreateForgottenToken();
        for (var attempt = 0; attempt < 20 && weakToken.IsAlive; attempt++)
        {
            GC.Collect();
            GC.WaitForPendingFinalizers();
            GC.Collect();
            Thread.Sleep(10);
        }

        Assert.False(weakToken.IsAlive);
        Assert.Equal(1, AbortCalls);
    }

    [Fact]
    public void TokenCannotCrossVoxelWorldOwnershipBoundary()
    {
        AbortCalls = 0;
        using var lease = CreateLease(CreateApi(
            blockWritePrepare: Pointer(Prepare),
            blockWriteAbort: Pointer(Abort)));
        var first = lease.CreateVoxelWorld((nint)0x1241);
        var second = lease.CreateVoxelWorld((nint)0x1242);
        using var token = first.PrepareWrite(44, Array.Empty<Sdk.VoxelBlockWriteEntry>());

        Assert.Throws<ArgumentException>(() => second.Abort(token));
        Assert.False(token.IsCompleted);
        Assert.Equal(1, lease.ActiveNativeTokenCount);
    }

    [Fact]
    public void SectionRevisionQueryPreservesNativePresenceAndRevision()
    {
        var api = CreateApi();
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);

        var result = world.QuerySectionRevision(new Sdk.VoxelSectionKey(-2, 15, 8));

        Assert.Equal(Sdk.VoxelPresence.Ready, result.Presence);
        Assert.Equal(27ul, result.SectionRevision);
    }

    /// <summary>
    /// 穷尽性断言：手写的 <c>VoxelErrorCode</c> 枚举与 <c>ContractStatuses</c> 必须覆盖
    /// <c>engine/abi/native-abi.json</c> 里的**每一条** voxel 错误码，数值与成员名都要与契约一致，
    /// 且不得多出契约之外的成员。契约追加错误码而这里漏改时，本用例必须变红。
    /// </summary>
    [Fact]
    public void EveryContractStatusHasAStableManagedErrorCode()
    {
        var (statusBase, codes) = LoadContractErrorCodes();

        for (var index = 0; index < codes.Count; index++)
        {
            var status = statusBase + index;
            var contractName = codes[index];
            Assert.True(
                Sdk.VoxelErrorCodeMap.TryMap(status, out var code),
                $"contract error code '{contractName}' (status {status}) has no named managed member");
            Assert.False(
                code == Sdk.VoxelErrorCode.Unknown,
                $"contract error code '{contractName}' (status {status}) collapsed into VoxelErrorCode.Unknown");
            Assert.Equal(status, Sdk.VoxelErrorCodeMap.ToStatus(code));
            Assert.Equal(Pascal(contractName), code.ToString());
        }

        var namedMembers = Enum.GetValues<Sdk.VoxelErrorCode>()
            .Where(member => member != Sdk.VoxelErrorCode.Unknown)
            .ToArray();
        Assert.Equal(codes.Count, namedMembers.Length);
    }

    /// <summary>
    /// 读取公共契约里的 <c>voxel.errorStatusBase</c> 与 <c>voxel.errorCodes</c>（唯一真值）。
    /// </summary>
    private static (int StatusBase, IReadOnlyList<string> Codes) LoadContractErrorCodes()
    {
        var definitionPath = LocateAbiDefinition();
        using var document = System.Text.Json.JsonDocument.Parse(File.ReadAllBytes(definitionPath));
        var voxel = document.RootElement.GetProperty("voxel");
        var statusBase = voxel.GetProperty("errorStatusBase").GetInt32();
        var codes = voxel.GetProperty("errorCodes")
            .EnumerateArray()
            .Select(entry => entry.GetString()!)
            .ToArray();

        Assert.NotEmpty(codes);
        return (statusBase, codes);
    }

    private static string LocateAbiDefinition()
    {
        var directory = new DirectoryInfo(AppContext.BaseDirectory);
        while (directory is not null)
        {
            var candidate = Path.Combine(directory.FullName, "engine", "abi", "native-abi.json");
            if (File.Exists(candidate))
            {
                return candidate;
            }

            directory = directory.Parent;
        }

        throw new FileNotFoundException(
            $"engine/abi/native-abi.json not found above {AppContext.BaseDirectory}");
    }

    private static string Pascal(string contractErrorCode)
        => string.Concat(contractErrorCode
            .Split('_', StringSplitOptions.RemoveEmptyEntries)
            .Select(part => char.ToUpperInvariant(part[0]) + part[1..]));

    [Fact]
    public void UnregisteredBlockTypeUsesStableStatus1051()
    {
        Assert.Equal(1051, (int)Sdk.VoxelErrorCode.UnregisteredBlockType);
        Assert.True(Sdk.VoxelErrorCodeMap.TryMap(1051, out var code));
        Assert.Equal(Sdk.VoxelErrorCode.UnregisteredBlockType, code);
        Assert.Equal(1051, Sdk.VoxelErrorCodeMap.ToStatus(code));
    }

    /// <summary>
    /// Unresolved 必须原样浮到托管层：既不塌缩成 Miss，也不塌缩成 Hit，而且携带 SectionKey。
    /// </summary>
    [Fact]
    public void RaycastSurfacesUnresolvedWithTheBlockingSectionKey()
    {
        var api = CreateApi(raycast: Pointer(Raycast));
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);

        var result = world.Raycast(new Sdk.VoxelRaycastRequest(
            new Sdk.VoxelWorldPoint(0.5f, 31.5f, 0.5f),
            new Sdk.VoxelWorldPoint(0f, -1f, 0f),
            64f,
            MaterialMaskSolid));

        Assert.Equal(Sdk.VoxelQueryResolution.Unresolved, result.Resolution);
        Assert.NotEqual(Sdk.VoxelQueryResolution.Miss, result.Resolution);
        Assert.NotEqual(Sdk.VoxelQueryResolution.Hit, result.Resolution);
        Assert.Equal(new Sdk.VoxelSectionKey(3, 1, -4), result.UnresolvedSection);
        Assert.Equal(0u, result.BlockId);
    }

    [Fact]
    public void SweepReturnsExplicitResolutionCollisionAndUnsignedBlockId()
    {
        var api = CreateApi(sweep: Pointer(Sweep));
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);

        var result = world.Sweep(new Sdk.VoxelSweepRequest(
            new Sdk.VoxelWorldPoint(0.5f, 2.5f, 0.5f),
            new Sdk.VoxelWorldPoint(0.5f, 0.9f, 0.5f),
            new Sdk.VoxelWorldPoint(0f, -4f, 0f),
            MaterialMaskSolid));

        Assert.Equal(Sdk.VoxelQueryResolution.Hit, result.Resolution);
        Assert.True(result.Collided);
        Assert.Equal(0.25f, result.TravelFraction);
        Assert.InRange(result.TravelFraction, 0f, 1f);
        Assert.Equal(0x80000123u, result.BlockId);
        Assert.Equal(new Sdk.VoxelWorldCoordinate(7, 9, -2), result.HitCell);
        Assert.Equal(1f, result.HitNormal.Y);
    }

    [Fact]
    public void OverlapWritesCallerBufferAndReportsTruncationWithTheActualCount()
    {
        var api = CreateApi(overlap: Pointer(Overlap));
        using var lease = CreateLease(api);
        var world = lease.CreateVoxelWorld((nint)0x1234);
        var hits = new Sdk.VoxelOverlapHit[2];

        var result = world.Overlap(
            new Sdk.VoxelOverlapRequest(
                new Sdk.VoxelWorldPoint(5f, 5.5f, 5.5f),
                new Sdk.VoxelWorldPoint(5f, 0.5f, 0.5f),
                MaterialMaskSolid),
            hits);

        Assert.Equal(Sdk.VoxelQueryResolution.Hit, result.Resolution);
        Assert.Equal(5u, result.ActualCount);
        Assert.True(result.Truncated);
        Assert.Equal(0x80000100u, hits[0].BlockId);
        Assert.Equal(0x80000101u, hits[1].BlockId);
        Assert.Equal(new Sdk.VoxelWorldCoordinate(1, 5, 6), hits[1].Cell);
    }

    /// <summary>契约 voxel.enums.material_mask：位 0 = Solid。</summary>
    private const uint MaterialMaskSolid = 1;

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static WeakReference CreateForgottenToken()
    {
        var lease = CreateLease(CreateApi(
            blockWritePrepare: Pointer(Prepare),
            blockWriteAbort: Pointer(Abort)));
        var world = lease.CreateVoxelWorld((nint)0x1243);
        var token = world.PrepareWrite(45, Array.Empty<Sdk.VoxelBlockWriteEntry>());
        lease.Dispose();
        return new WeakReference(token);
    }

    private static NativeEngineLease CreateLease(NativeEngineLoader.RootApi api)
        => new(0, api, "fake", "build", "abi", "sha", api.Ping);

    private static NativeEngineLoader.RootApi CreateApi(
        nint blockReadCell = 0,
        nint blockReadBox = 0,
        nint blockWritePrepare = 0,
        nint blockWriteCommit = 0,
        nint blockWriteAbort = 0,
        nint raycast = 0,
        nint sweep = 0,
        nint overlap = 0)
        => new()
        {
            AbiVersion = AbiConstants.AbiVersion,
            StructSize = 304,
            AbiHash = new byte[32],
            BuildId = new byte[16],
            Ping = 1,
            CreateClrHost = 1,
            ClrHostCall = 1,
            DestroyClrHost = 1,
            BlockReadCell = blockReadCell,
            BlockReadBox = blockReadBox,
            BlockReadColumn = blockReadBox,
            BlockWritePrepare = blockWritePrepare,
            BlockWriteCommit = blockWriteCommit,
            BlockWriteAbort = blockWriteAbort,
            SectionRevisionQuery = Pointer(Revision),
            Raycast = raycast,
            Sweep = sweep,
            Overlap = overlap,
        };

    private static nint Pointer(Delegate callback)
        => Marshal.GetFunctionPointerForDelegate(callback);

    private static int ReadCellImpl(nint _, nint __, nint result)
    {
        Marshal.StructureToPtr(new VoxelBlockReadCellResult
        {
            Presence = VoxelPresence.Pending,
            HasBlockId = 0,
            Reserved = new byte[3],
            BlockId = 0,
            SectionRevision = 17,
        }, result, false);
        return 0;
    }

    private static int ReadBatchImpl(nint _, nint __, nint results, uint resultCapacity, nint resultCount, nint segments, uint segmentCapacity, nint segmentCount, nint truncated)
    {
        Marshal.WriteInt32(resultCount, 2);
        Marshal.WriteInt32(segmentCount, 1);
        Marshal.WriteByte(truncated, 1);
        if (resultCapacity > 0)
        {
            Marshal.StructureToPtr(new VoxelBlockReadResult
            {
                Presence = VoxelPresence.Ready,
                HasBlockId = 1,
                Reserved = new byte[3],
                BlockId = 0x80000123,
                SectionRevision = 22,
            }, results, false);
        }

        if (segmentCapacity > 0)
        {
            Marshal.StructureToPtr(new VoxelSectionSegment
            {
                SectionKey = new VoxelSectionKey { X = 1, Y = 2, Z = 3, Reserved = new byte[3] },
                Presence = VoxelPresence.Ready,
                SectionRevision = 22,
                FirstResult = 0,
                ResultCount = 2,
            }, segments, false);
        }

        return 0;
    }

    private static int PrepareImpl(nint _, ulong __, nint ___, uint ____, out nint token)
    {
        token = (nint)0x99;
        return 0;
    }

    private static int PrepareWaitsForLeaseDisposeImpl(nint _, ulong __, nint ___, uint ____, out nint token)
    {
        token = (nint)0x9a;
        PrepareEntered?.Set();
        if (LeaseDisposed is null || !LeaseDisposed.Wait(TimeSpan.FromSeconds(5)))
        {
            throw new TimeoutException("The lease was not disposed while native prepare was in flight.");
        }

        return 0;
    }

    private static int CommitImpl(nint _, nint __, nint receipts, uint capacity, nint count)
    {
        Marshal.WriteInt32(count, 1);
        if (capacity > 0)
        {
            Marshal.StructureToPtr(new VoxelWriteReceipt
            {
                SectionKey = new VoxelSectionKey { X = 1, Y = 2, Z = 3, Reserved = new byte[3] },
                UpToSectionRevision = 10,
                WorldRevision = 99,
            }, receipts, false);
        }

        return 0;
    }

    private static int AbortImpl(nint _, nint __)
    {
        Interlocked.Increment(ref AbortCalls);
        return 0;
    }

    /// <summary>Unresolved：携带挡路的 SectionKey，且不带任何命中数据。</summary>
    private static int RaycastImpl(nint _, nint request, nint result)
    {
        var incoming = Marshal.PtrToStructure<VoxelRaycastRequest>(request);
        Assert.Equal(1u, incoming.MaterialMask);
        Assert.Equal(-1f, incoming.Direction.Y);
        Marshal.StructureToPtr(new VoxelRaycastResult
        {
            Resolution = VoxelQueryResolution.Unresolved,
            UnresolvedSection = new VoxelSectionKey { X = 3, Y = 1, Z = -4, Reserved = new byte[3] },
            HitCell = new VoxelWorldCoordinate { X = 0, Y = 0, Z = 0 },
            BlockId = 0,
            HitPoint = default,
            HitNormal = default,
            TravelDistance = 0,
        }, result, false);
        return 0;
    }

    private static int SweepImpl(nint _, nint request, nint result)
    {
        var incoming = Marshal.PtrToStructure<VoxelSweepRequest>(request);
        // 形状按值内联：center 即位姿，没有单独的位姿指针。
        Assert.Equal(2.5f, incoming.Center.Y);
        Assert.Equal(0.5f, incoming.HalfExtents.X);
        Marshal.StructureToPtr(new VoxelSweepResult
        {
            Resolution = VoxelQueryResolution.Hit,
            Collided = 1,
            Reserved = new byte[3],
            TravelFraction = 0.25f,
            UnresolvedSection = new VoxelSectionKey { X = 0, Y = 0, Z = 0, Reserved = new byte[3] },
            HitCell = new VoxelWorldCoordinate { X = 7, Y = 9, Z = -2 },
            BlockId = 0x80000123,
            HitPoint = new VoxelWorldPoint { X = 1, Y = 2, Z = 3 },
            HitNormal = new VoxelWorldPoint { X = 0, Y = 1, Z = 0 },
        }, result, false);
        return 0;
    }

    /// <summary>容量 2 罩 5 格：写满调用方缓冲，并回报 truncated 与实际总数。</summary>
    private static int OverlapImpl(nint _, nint __, nint hits, uint capacity, nint result)
    {
        var hitSize = Marshal.SizeOf<VoxelOverlapHit>();
        for (var index = 0; index < (int)capacity; index++)
        {
            Marshal.StructureToPtr(new VoxelOverlapHit
            {
                Cell = new VoxelWorldCoordinate { X = index, Y = 5, Z = 6 },
                BlockId = (uint)(0x80000100 + index),
            }, hits + index * hitSize, false);
        }

        Marshal.StructureToPtr(new VoxelOverlapResult
        {
            Resolution = VoxelQueryResolution.Hit,
            UnresolvedSection = new VoxelSectionKey { X = 0, Y = 0, Z = 0, Reserved = new byte[3] },
            ActualCount = 5,
            Truncated = 1,
            Reserved = new byte[3],
        }, result, false);
        return 0;
    }

    private static int RevisionImpl(nint _, nint __, nint result)
    {
        Marshal.StructureToPtr(new VoxelSectionRevisionResult
        {
            Presence = VoxelPresence.Ready,
            Reserved = new byte[4],
            SectionRevision = 27,
        }, result, false);
        return 0;
    }
}
