Unity replicates the 3DRT wind field system of God of War 4
SabreGodLY
SabreGodLY
Looking for a new job
144 people upvoted the article
​
Table of contents
Collapse
Preface
Get ready
Start to implement
ComputeShader
Create a Buffer
The effect is realized
Offset of wind data
Wind diffusion simulation
Wind speed calculation and storage for wind engines
Advection and convection simulation of wind
The final wind information for each channel is stored in a single 3D RT
The running effect on the mobile phone
Finally, add 100 million things
reference
Preface
I saw it some time agoGDC19There is a sharing of God of War 4, which is quite interesting about the 3D wind field, so I thought about replicating one in Unity

It is highly recommended to watch the GDC sharing video and some PPT sharing explanations below before reading this article, because this article focuses on specific implementations, and will not go into detail about the content shared by GDC


Si Hu Hu: Wind Simulation in 'God of War' (GDC2019)
205 Agree · 11 Reviews Article

(PS: I made a version to use it later.)Unity ECS systemThe wind simulation version can be confidently implemented on the mobile terminal. Because the overall idea is to distribute the logical calculations that can only be achieved using Compute ShaderJob SystemWithBurstand optimized the Sync Points that may occur. This article mainly introduces the implementation method of wind simulation, and if you are interested in the specific content, it is recommended to read the repository source code at the end of the article)

Get ready
This is implemented using Unity2021LTS version, URP version 12.1.6

The URP version doesn't have much impact, just uses URP's RenderFeature to mobilize the rendering of wind engines, but why use such a new Unity version?

At first, it was done on the 2019.4.16 version, but the old version had a very painful problem -

We know URP'sCommandBufferIt is stateless, that is, after each frame is rendered through the context, the CommandBuffer starting the next frame does not know what RT data the previous frame holds. However, the 3DRT of our wind farm simulation needs to be recycled and frame-to-frame, so we need to create an external class to manage this part of the RT declaration and ComputeShader instructions.

但是很疑惑的是2019版本的Unity创建了RT之后在ComputeShader也传递了，全局Shader声明也做了，可是就是会收到空RT，而查看Profiler的内存数据你能发现这个RT实际上是已经声明了。

然后很骚的是你如果双击查看了这个RT之后这个RT在这次Unity重启之前都可以传递到ComputeShader和Shader中了。。。

代码原封不动，升级到2021的LTS版本后就一点问题都没有。我确信这个问题是Unity的Bug（碎碎念

开始实现
ComputeShader
战神中采用的风力存储方式是使用3个通道x2份32x16x32的3D RenderTextureBuffer+1份结合了所有通道的3D RenderTexture Buffer，至于这么做的原因后面会介绍

在这之前，用一小部分内容来介绍一下ComputeShader


图1 一个在Unity中创建的基础ComputeShader

图2 一部分源码内容，此处为在C#端调用Dispatch执行ComputeShader的语句

图3 Dispatch代表线程组的数量设置，而numthread代表每个线程组中的线程数量的设置
对于一个新创建的cs，需要理解几个基本概念

kernel：计算核，代表的是你在这个cs中注册的计算函数，也即是cs功能的入口，在c#等脚本调用，可以存在多个
numthread：单个线程组的大小，表示一个线程组内有多少个线程，例如图1中的就是这个线程组内有8* 8 * 1 = 64个线程（Thread），图3里头就是10 * 8 * 3 = 240个线程
dispatch：线程组阵列，表示这个ComputeShader在C#处调用时会创建多少个线程组
其中图内的m_WindBrandX，m_WindBrandY，m_WindBrandZ分别为32，16和32，所以上面Dispatch中的xyz就分别是8，4和8总共256个线程组（ThreadGroup）

（不过战神4中，出于对ComputeShader的并行效率考虑，将y轴和z轴对换，然后在ComputeShader中避免了在设置numthreads时保证z轴为1，兼容了一些cs版本低的设备。这里只是为了实现效果这一步就跳过了）

创建Buffer
(起初我在复刻的时候还傻乎乎用的Texture3D。。。)

实际上我们需要创建的是RenderTexture，并将维度设置成3D的，width和height仍然代表的是x和y维度上的大小，depth为z维度的大小

战神的分享中提到了将通道拆解出来进行处理可以提高运行速度，同时可以使用更高的精度满足后面的原子计算的需求，所以这里需要创建6份3DRT，以及最终供Shader采样的1份最终的RT。通道的RT格式在PPT中也有提到，使用的是带符号的32位整型SInt。使用这个通道的好处就是带正负，不用我们像做NormalMap的时候要做个映射。最终的RT不需要再区分通道，使用的是RGBA32，即前面拆分通道的计算结果合并到这张RT上。别看这个格式虽然很吓人，但是这个3DRenderTexture非常小，应该吃得消（吧）


图4 RT的声明，6份单通道的RT Buffer和1份最终用于采样的RT Buffer

图5 RT格式和RT创建
效果实现
下面开始介绍战神4的3D风场的实现流程

按照效果的实现流程，可以将流程分为以下的内容

风力数据的偏移存储
风的扩散模拟
风力发动机的风力速度计算和存储
风的平流和对流模拟
各个通道的最终风力信息存储到1份3D RT中
至于为什么不是从风力发动机的计算开始呢？我不管，听我的（

风力数据的偏移
风力的计算单通道的RT存储数据的形式，只有在输出的时候才合并出一份混合RGB通道的3DRenderTexture。

在开始计算第一步的时候就需要有个概念，就是上一帧的风力数据是会留到下一帧并参与模拟计算和效果表现的。

而这一步，就是为了纠正风力中心对象的移动后导致的风力信息的记录偏差。

假设我们的风力记录的中心对象O是我们当前在操作的玩家，当前帧下O的世界空间下的坐标我们记为 
 ，这一帧下我们的风力信息都是在这个坐标下进行计算和存储的。

假设下一帧时O往x轴正方向多走了1米（RT中1个像素代表1米空间的数据），下一帧的风力记录的中心点是不是就应该是 
 了呢？因此，对于下一帧的而言，所有的风力信息都要往x轴反方向偏移1个像素，才能准确继承每个世界空间下的风力信息。

按照这个规律总结，我们可以实现下面的偏移算法和对应的ComputeShader

注意我们在RT中的存储规律是1立方厘米代表一个像素，所以针对RT的操作都需要转换为整数


图6 之所以写from这个玩意儿时为了方便调试，保证了效果之后其实留不留都可以

（PS:这里我遇到过一个非常坑爹的bug。看代码可以留意到我这边将各个通道的偏移量分开传入ComputeShader中，这么做的原因是我使用int[]数组也好Vector3转Int也好都会出现数据无法存储的情况。。。如果有知道原因的hxd还不吝赐教。。。）

风的扩散模拟
视频中主讲人自己提到了，PPT里也提到了，对于扩散模拟你只需要把这个效果看作一个blur（模糊）就好

那我们就按模糊的方式来做就好啦


图7
#include "WindSimulationInclude.hlsl"
#pragma kernel CSMain

Texture3D<int> WindBufferInput;
RWTexture3D<int> WindBufferOutput;

uniform int3 VolumeSizeMinusOne;
uniform float DiffusionForce;

#define N 4
groupshared float m_Cache[N*N*N];

[numthreads(N,N,N)]
void CSMain (int3 dispatchThreadID : SV_DispatchThreadID, int3 groupThreadID : SV_GroupThreadID)
{
    // 采样int格式的Volume纹理,并转换格式
    float windInput = PackIntToFloat(WindBufferInput[dispatchThreadID.xyz].r);
    // 计算节点Groupid
    int cacheIndex = groupThreadID.x + groupThreadID.y * 4 + groupThreadID.z * 16;
    m_Cache[cacheIndex] = windInput;
    GroupMemoryBarrierWithGroupSync();
    // 判断各个方向上的越界问题和数据cache获取逻辑
    float xr = 0;
    float xl = 0;
    float yr = 0;
    float yl = 0;
    float zr = 0;
    float zl = 0;
    // X轴
    if(groupThreadID.x < N - 1)
    {
        int3 gtID = groupThreadID + int3(1, 0, 0);
        xr = m_Cache[gtID.x + gtID.y * 4 + gtID.z * 16];
    }
    else
    {
        int tIDx = min(dispatchThreadID.x + 1, VolumeSizeMinusOne.x);
        xr = PackIntToFloat(WindBufferInput[int3(tIDx, dispatchThreadID.y, dispatchThreadID.z)].r);
    }
    if(groupThreadID.x != 0)
    {
        int3 gtID = groupThreadID + int3(-1, 0, 0);
        xl = m_Cache[gtID.x + gtID.y * 4 + gtID.z * 16];
    }
    else
    {
        int tIDx = max(dispatchThreadID.x - 1, 0);
        xl = PackIntToFloat(WindBufferInput[int3(tIDx, dispatchThreadID.y, dispatchThreadID.z)].r);
    }
    // Y轴Z轴同理，省略
    ......
    // 最终合并diffusion模拟
    float finalData = xr + xl + yr + yl + zr + zl - windInput * 6;
    finalData = finalData * DiffusionForce + windInput;
    WindBufferOutput[dispatchThreadID.xyz] = PackFloatToInt(finalData);
}
其中，PackIntToFloat和PackFloatToInt是我自己封装的方法，实际上就是实现分享PPT中的数值变换而已


图8 把红框的部分抽出来，然后做了个复原成float的函数
风力发动机的风力速度计算和存储
这块算是模拟的核心部分，需要我们记录场景内的风力发动机并存储风速信息到单通道的RT Buffer中

战神4的分享PPT中直接涵盖了3个风力发动机的算法，可以先用这三个进行模拟复刻


图9 官方分享中涉及到的发动机类型的算法
我们需要在C#端编写发动机WindMotor，并且在我们的模拟脚本中持有并管理所有的Motor的生命周期和数据更新。

我们需要使用前面计算出来的模糊数据作为基础，在计算出风力发动机的数值之后与其叠加。

// 风力发动机类代码（部分）
public enum MotorType
{
    Directional,
    Omni,
    Vortex,
}

public struct MotorDirectional
{
    public Vector3 position;
    public float radiusSq;
    public Vector3 force;
}

public struct MotorOmni
{
    public Vector3 position;
    public float radiusSq;
    public float force;
}

public struct MotorVortex
{
    public Vector3 position;
    public Vector3 axis;
    public float radiusSq;
    public float force;
}

public class WindMotor : MonoBehaviour
{
    public MotorType MotorType;
    public MotorDirectional motorDirectional;
    public MotorOmni motorOmni;
    public MotorVortex motorVortex;

    private static MotorDirectional emptyMotorDirectional = new MotorDirectional();
    private static MotorOmni emptyMotorOmni = new MotorOmni();
    private static MotorVortex emptyMotorVortex = new MotorVortex();

    public static MotorDirectional GetEmptyMotorDirectional()
    {
        return emptyMotorDirectional;
    }
    public static MotorOmni GetEmptyMotorOmni()
    {
        return emptyMotorOmni;
    }
    public static MotorVortex GetEmptyMotorVortex()
    {
        return emptyMotorVortex;
    }
    
    /// <summary>
    /// 创建风力发电机的时间，以Time.fixedTime为准
    /// </summary>
    private float m_CreateTime;
    public bool Loop = true;
    public float LifeTime = 5f;
    [Range(0.001f, 100f)]
    public float Radius = 1f;
    public AnimationCurve RadiusCurve = AnimationCurve.Linear(1, 1, 1, 1);
    public Vector3 Asix = Vector3.up;
    public float Force = 1f;
    public AnimationCurve ForceCurve = AnimationCurve.Linear(1, 1, 1, 1);
    public float Duration = 0f;

    private Vector3 m_prePosition = Vector3.zero;
    #region BasicFunction

    private void Start()
    {
        m_CreateTime = Time.fixedTime;
    }

    private void OnEnable()
    {
        WindManager.Instance.AddWindMotor(this);
        m_CreateTime = Time.fixedTime;
    }

    private void OnDisable()
    {
        WindManager.Instance.RemoveWindMotor(this);
    }

    private void OnDestroy()
    {
        WindManager.Instance.RemoveWindMotor(this);
    }
    #endregion
    
    #region MainFunction
    /// <summary>
    /// 检查声明周期结束
    /// </summary>
    /// <param name="duration"></param>
    void CheckMotorDead()
    {
        float duration = Time.fixedTime - m_CreateTime;
        if (duration > LifeTime)
        {
            if (Loop)
            {
                m_CreateTime = Time.fixedTime;
            }
            else
            {
                m_CreateTime = 0f;
                WindPool.Instance.PushWindMotor(this.gameObject);
            }
        }
    }
    #endregion
    
    #region UpdateForceAndOtherProperties
    /// <summary>
    /// 调用的时候才更新风的参数
    /// </summary>
    public void UpdateWindMotor()
    {
        switch (MotorType)
        {
            case MotorType.Directional:
                UpdateDirectionalWind();
                break;
            case MotorType.Omni:
                UpdateOmniWind();
                break;
            case MotorType.Vortex:
                UpdateVortexWind();
                break;
        }
    }
    private void UpdateDirectionalWind()
    {
        float duration = Time.fixedTime - m_CreateTime;
        float timePerc = duration / LifeTime;
        Duration = timePerc;
        float radius = Radius * RadiusCurve.Evaluate(timePerc);
        motorDirectional = new MotorDirectional()
        {
            position = transform.position,
            radiusSq = radius * radius,
            force = transform.forward * ForceCurve.Evaluate(timePerc) * Force
        };
        CheckMotorDead();
    }

    private void UpdateOmniWind()
    {
        float duration = Time.fixedTime - m_CreateTime;
        float timePerc = duration / LifeTime;
        Duration = timePerc;
        float radius = Radius * RadiusCurve.Evaluate(timePerc);
        motorOmni = new MotorOmni()
        {
            position = transform.position,
            radiusSq = radius * radius,
            force = ForceCurve.Evaluate(timePerc) * Force
        };
        CheckMotorDead();
    }

    private void UpdateVortexWind()
    {
        float duration = Time.fixedTime - m_CreateTime;
        float timePerc = duration / LifeTime;
        Duration = timePerc;
        float radius = Radius * RadiusCurve.Evaluate(timePerc);
        motorVortex = new MotorVortex()
        {
            position = transform.position,
            axis = Vector3.Normalize(Asix),
            radiusSq = radius * radius,
            force = ForceCurve.Evaluate(timePerc) * Force
        };
        CheckMotorDead();
    }
    #endregion
}

//风力发动机管理逻辑和ComputeShader数据传输逻辑（部分）
void DoRenderWindVelocityData(int form)
    {
        if (MotorsSpeedCS != null && BufferExchangeCS != null)
        {
            m_DirectionalMotorList.Clear();
            m_OmniMotorList.Clear();
            m_VortexMotorList.Clear();

            int directionalMotorCount = 0;
            int omniMotorCount = 0;
            int vortexMotorCount = 0;
            foreach (WindMotor motor in m_MotorList)
            {
                // 更新风力发动机数据
                motor.UpdateWindMotor();
                switch (motor.MotorType)
                {
                    case MotorType.Directional:
                        if (directionalMotorCount < MAXMOTOR)
                        {
                            m_DirectionalMotorList.Add(motor.motorDirectional);
                            directionalMotorCount++;
                        }
                        break;
                    case MotorType.Omni:
                        if (omniMotorCount < MAXMOTOR)
                        {
                            m_OmniMotorList.Add(motor.motorOmni);
                            omniMotorCount++;
                        }
                        break;
                    case MotorType.Vortex:
                        if (vortexMotorCount < MAXMOTOR)
                        {
                            m_VortexMotorList.Add(motor.motorVortex);
                            vortexMotorCount++;
                        }
                        break;
                }
            }
            // 往列表数据中插入空的发动机数据
            if (directionalMotorCount < MAXMOTOR)
            {
                MotorDirectional motor = WindMotor.GetEmptyMotorDirectional();
                for (int i = directionalMotorCount; i < MAXMOTOR; i++)
                {
                    m_DirectionalMotorList.Add(motor);
                }
            }
            if (omniMotorCount < MAXMOTOR)
            {
                MotorOmni motor = WindMotor.GetEmptyMotorOmni();
                for (int i = omniMotorCount; i < MAXMOTOR; i++)
                {
                    m_OmniMotorList.Add(motor);
                }
            }
            if (vortexMotorCount < MAXMOTOR)
            {
                MotorVortex motor = WindMotor.GetEmptyMotorVortex();
                for (int i = vortexMotorCount; i < MAXMOTOR; i++)
                {
                    m_VortexMotorList.Add(motor);
                }
            }

            m_DirectionalMotorBuffer.SetData(m_DirectionalMotorList);
            MotorsSpeedCS.SetBuffer(m_MotorSpeedKernel, m_DirectionalMotorBufferId, m_DirectionalMotorBuffer);
            m_OmniMotorBuffer.SetData(m_OmniMotorList);
            MotorsSpeedCS.SetBuffer(m_MotorSpeedKernel, m_OmniMotorBufferId, m_OmniMotorBuffer);
            m_VortexMotorBuffer.SetData(m_VortexMotorList);
            MotorsSpeedCS.SetBuffer(m_MotorSpeedKernel, m_VortexMotorBufferId, m_VortexMotorBuffer);

            MotorsSpeedCS.SetFloat(m_DirectionalMotorBufferCountId, directionalMotorCount);
            MotorsSpeedCS.SetFloat(m_OmniMotorBufferCountId, omniMotorCount);
            MotorsSpeedCS.SetFloat(m_VortexMotorBufferCountId, vortexMotorCount);
            MotorsSpeedCS.SetVector(m_VolumePosOffsetId, m_OffsetPos);
            
            var formRTR = form == 1 ? m_WindBufferChannelR1 : m_WindBufferChannelR2;
            var formRTG = form == 1 ? m_WindBufferChannelG1 : m_WindBufferChannelG2;
            var formRTB = form == 1 ? m_WindBufferChannelB1 : m_WindBufferChannelB2;
            var toRTR = form == 1 ? m_WindBufferChannelR2 : m_WindBufferChannelR1;
            var toRTG = form == 1 ? m_WindBufferChannelG2 : m_WindBufferChannelG1;
            var toRTB = form == 1 ? m_WindBufferChannelB2 : m_WindBufferChannelB1;
            
            MotorsSpeedCS.SetTexture(m_MotorSpeedKernel, m_WindBufferInputXID, formRTR);
            MotorsSpeedCS.SetTexture(m_MotorSpeedKernel, m_WindBufferInputYID, formRTG);
            MotorsSpeedCS.SetTexture(m_MotorSpeedKernel, m_WindBufferInputZID, formRTB);
            MotorsSpeedCS.SetTexture(m_MotorSpeedKernel, m_WindBufferOutputXID, toRTR);
            MotorsSpeedCS.SetTexture(m_MotorSpeedKernel, m_WindBufferOutputYID, toRTG);
            MotorsSpeedCS.SetTexture(m_MotorSpeedKernel, m_WindBufferOutputZID, toRTB);
            MotorsSpeedCS.Dispatch(m_MotorSpeedKernel, m_WindBrandX / 8, m_WindBrandY / 8, m_WindBrandZ);
            // 清除旧Buffer
            BufferExchangeCS.SetTexture(m_ClearBufferKernel, m_WindBufferOutputXID, formRTR);
            BufferExchangeCS.SetTexture(m_ClearBufferKernel, m_WindBufferOutputYID, formRTG);
            BufferExchangeCS.SetTexture(m_ClearBufferKernel, m_WindBufferOutputZID, formRTB);
            BufferExchangeCS.Dispatch(m_ClearBufferKernel, m_WindBrandX / 4, m_WindBrandY / 4, m_WindBrandZ / 4);
        }
    }

// ComputeShader相关部分代码
#include "WindSimulationInclude.hlsl"
#pragma kernel WindVolumeRenderMotorCS

StructuredBuffer<MotorDirectional> DirectionalMotorBuffer;
StructuredBuffer<MotorOmni> OmniMotorBuffer;
StructuredBuffer<MotorVortex> VortexMotorBuffer;
uniform float DirectionalMotorBufferCount;
uniform float OmniMotorBufferCount;
uniform float VortexMotorBufferCount;
// 风力坐标计算的采样偏移
uniform float3 VolumePosOffset;



Texture3D<int> WindBufferInputX;
Texture3D<int> WindBufferInputY;
Texture3D<int> WindBufferInputZ;
RWTexture3D<int> WindBufferOutputX;
RWTexture3D<int> WindBufferOutputY;
RWTexture3D<int> WindBufferOutputZ;

// 根据传入的风力发动机的buffer，覆盖对应的扩散风的信息
[numthreads(8,8,1)]
void WindVolumeRenderMotorCS (uint3 id : SV_DispatchThreadID)
{
    float3 cellPosWS = id.xyz + VolumePosOffset;
    
    float3 velocityWS = 0;
    velocityWS.x = PackIntToFloat(WindBufferInputX[id.xyz].r);
    velocityWS.y = PackIntToFloat(WindBufferInputY[id.xyz].r);
    velocityWS.z = PackIntToFloat(WindBufferInputZ[id.xyz].r);
    // 根据已有的风力数量来叠加风力信息
    if(DirectionalMotorBufferCount > 0)
    {
        for(int i = 0; i < DirectionalMotorBufferCount; i++)
        {
            ApplyMotorDirectional(cellPosWS, DirectionalMotorBuffer[i], velocityWS);
        }
    }
    if(OmniMotorBufferCount > 0)
    {
        for(int i = 0; i < OmniMotorBufferCount; i++)
        {
            ApplyMotorOmni(cellPosWS, OmniMotorBuffer[i], velocityWS);
        }
    }
    if(VortexMotorBufferCount > 0)
    {
        for(int i = 0; i < VortexMotorBufferCount; i++)
        {
            ApplyMotorVortex(cellPosWS, VortexMotorBuffer[i], velocityWS);
        }
    }

    WindBufferOutputX[id.xyz] = PackFloatToInt(velocityWS.x);
    WindBufferOutputY[id.xyz] = PackFloatToInt(velocityWS.y);
    WindBufferOutputZ[id.xyz] = PackFloatToInt(velocityWS.z);
}
风的平流和对流模拟

图10 平流对流的分享PPT内容
这一步的整体思路就是计算出风力方向后，根据风力的强度计算出在这个方向上要移动的像素距离，再根据这个移动距离将速度信息移动到目标的像素格子上。

而对流模拟的话，战神的做法就是，简单的将平流模拟的系数取反。。。

也不管人家处于什么想法，反正效果可以接受就好啦

#pragma kernel CSMain
#pragma kernel CSMain2
// 传入3个不同方向的风力纹理
Texture3D<int> WindBufferInputX;
Texture3D<int> WindBufferInputY;
Texture3D<int> WindBufferInputZ;
// 这个纹理代表这次要处理的轴向
Texture3D<int> WindBufferTarget;
RWTexture3D<int> WindBufferOutput;

uniform int3 VolumeSizeMinusOne;
uniform float AdvectionForce;

#define N 4
// 平流模拟正向平流
// 将一个像素上的风力传递到目标像素上
[numthreads(N,N,N)]
void CSMain (int3 dispatchThreadID : SV_DispatchThreadID)
{
    // 抽取目标轴向纹理数据
    float targetData = PackIntToFloat(WindBufferTarget[dispatchThreadID.xyz].r);
    // 抽取三个方向的风力数据
    int3 windDataInt = int3(WindBufferInputX[dispatchThreadID.xyz].r, WindBufferInputY[dispatchThreadID.xyz].r,
                            WindBufferInputZ[dispatchThreadID.xyz].r);
    float3 advectionData = windDataInt * AdvectionForce * FXDPT_SIZE_R;
    int3 moveCell = (int3)(floor(advectionData + dispatchThreadID));
    // 指定当前格子和周围格子的偏移比例
    float3 offsetNeb = frac(advectionData);
    float3 offsetOri = 1.0 - offsetNeb;

    // 根据风向偏移到指定的格子后，开始计算各个方向的平流
    if(all(moveCell >= 0 && moveCell <= VolumeSizeMinusOne))
    {
        float adData = offsetOri.x * offsetOri.y * offsetOri.z * targetData;
        InterlockedAdd(WindBufferOutput[moveCell.xyz], PackFloatToInt(adData));
    }
    // 目标中心x+1
    int3 tempCell = moveCell + int3(1, 0, 0);
    if(all(tempCell >= 0 && tempCell <= VolumeSizeMinusOne))
    {
        float adData = offsetNeb.x * offsetOri.y * offsetOri.z * targetData;
        InterlockedAdd(WindBufferOutput[tempCell.xyz], PackFloatToInt(adData));
    }
    // 目标中心z+1(但战神中的yz是反转的,因此整理完之后要将其归位置,这里实际上就是y+1)
    // 不偏移的部分取advectionData分量,否则取advectionDataFrac分量
    tempCell = moveCell + int3(0, 1, 0);
    if(all(tempCell >= 0 && tempCell <= VolumeSizeMinusOne))
    {
        float adData = offsetOri.x * offsetNeb.y * offsetOri.z * targetData;
        InterlockedAdd(WindBufferOutput[tempCell.xyz], PackFloatToInt(adData));
    }
    
    tempCell = moveCell + int3(1, 1, 0);
    if(all(tempCell >= 0 && tempCell <= VolumeSizeMinusOne))
    {
        float adData = offsetNeb.x * offsetNeb.y * offsetOri.z * targetData;
        InterlockedAdd(WindBufferOutput[tempCell.xyz], PackFloatToInt(adData));
    }

    tempCell = moveCell + int3(0, 0, 1);
    if(all(tempCell >= 0 && tempCell <= VolumeSizeMinusOne))
    {
        float adData = offsetOri.x * offsetOri.y * offsetNeb.z * targetData;
        InterlockedAdd(WindBufferOutput[tempCell.xyz], PackFloatToInt(adData));
    }

    tempCell = moveCell + int3(1, 0, 1);
    if(all(tempCell >= 0 && tempCell <= VolumeSizeMinusOne))
    {
        float adData = offsetNeb.x * offsetOri.y * offsetNeb.z * targetData;
        InterlockedAdd(WindBufferOutput[tempCell.xyz], PackFloatToInt(adData));
    }

    tempCell = moveCell + int3(0, 1, 1);
    if(all(tempCell >= 0 && tempCell <= VolumeSizeMinusOne))
    {
        float adData = offsetOri.x * offsetNeb.y * offsetNeb.z * targetData;
        InterlockedAdd(WindBufferOutput[tempCell.xyz], PackFloatToInt(adData));
    }

    tempCell = moveCell + int3(1, 1, 1);
    if(all(tempCell >= 0 && tempCell <= VolumeSizeMinusOne))
    {
        float adData = offsetNeb.x * offsetNeb.y * offsetNeb.z * targetData;
        InterlockedAdd(WindBufferOutput[tempCell.xyz], PackFloatToInt(adData));
    }
}

// 平流模拟反向平流
// 这个模拟的是从反平流方向的一个3x3区域的风力信息传递到目标像素上
[numthreads(N,N,N)]
void CSMain2 (int3 dispatchThreadID : SV_DispatchThreadID)
{
    // 抽取三个方向的风力数据
    float3 windDataInt = PackIntToFloat(int3(WindBufferInputX[dispatchThreadID.xyz].r, WindBufferInputY[dispatchThreadID.xyz].r,
                            WindBufferInputZ[dispatchThreadID.xyz].r));
    float3 advectionData = windDataInt * -AdvectionForce;
    int3 moveCell = (int3)(floor(advectionData + dispatchThreadID));
    // 指定当前格子和周围格子的偏移比例
    float3 offsetNeb = frac(advectionData);
    float3 offsetOri = 1.0 - offsetNeb;
    // 抽取目标轴向纹理数据
    float targetData = PackIntToFloat(WindBufferTarget[moveCell.xyz].r);
    targetData *= offsetOri.x * offsetOri.y * offsetOri.z;

    int3 tempPos1 = moveCell.xyz + int3(1, 0, 0);
    float targetDataX1 = PackIntToFloat(WindBufferTarget[tempPos1.xyz].r);
    targetDataX1 *= offsetNeb.x * offsetOri.y * offsetOri.z;

    int3 tempPos2 = moveCell.xyz + int3(0, 1, 0);
    float targetDataY1 = PackIntToFloat(WindBufferTarget[tempPos2.xyz].r);
    targetDataY1 *= offsetOri.x * offsetNeb.y * offsetOri.z;

    int3 tempPos3 = moveCell.xyz + int3(1, 1, 0);
    float targetDataX1Y1 = PackIntToFloat(WindBufferTarget[tempPos3.xyz].r);
    targetDataX1Y1 *= offsetNeb.x * offsetNeb.y * offsetOri.z;

    int3 tempPos4 = moveCell.xyz + int3(0, 0, 1);
    float targetDataZ1 = PackIntToFloat(WindBufferTarget[tempPos4.xyz].r);
    targetDataZ1 *= offsetOri.x * offsetOri.y * offsetNeb.z;

    int3 tempPos5 = moveCell.xyz + int3(1, 0, 1);
    float targetDataX1Z1 = PackIntToFloat(WindBufferTarget[tempPos5.xyz].r);
    targetDataX1Z1 *= offsetNeb.x * offsetOri.y * offsetNeb.z;

    int3 tempPos6 = moveCell.xyz + int3(0, 1, 1);
    float targetDataY1Z1 = PackIntToFloat(WindBufferTarget[tempPos6.xyz].r);
    targetDataY1Z1 *= offsetOri.x * offsetNeb.y * offsetNeb.z;

    int3 tempPos7 = moveCell.xyz + int3(1, 1, 1);
    float targetDataX1Y1Z1 = PackIntToFloat(WindBufferTarget[tempPos7.xyz].r);
    targetDataX1Y1Z1 *= offsetNeb.x * offsetNeb.y * offsetNeb.z;

    if(all(moveCell >= 0 && moveCell <= VolumeSizeMinusOne))
    {
        InterlockedAdd(WindBufferOutput[moveCell.xyz], -PackFloatToInt(targetData));
    }
    if(all(tempPos1 >= 0 && tempPos1 <= VolumeSizeMinusOne))
    {
        InterlockedAdd(WindBufferOutput[tempPos1.xyz], -PackFloatToInt(targetDataX1));
    }
    if(all(tempPos2 >= 0 && tempPos2 <= VolumeSizeMinusOne))
    {
        InterlockedAdd(WindBufferOutput[tempPos2.xyz], -PackFloatToInt(targetDataY1));
    }
    if(all(tempPos3 >= 0 && tempPos3 <= VolumeSizeMinusOne))
    {
        InterlockedAdd(WindBufferOutput[tempPos3.xyz], -PackFloatToInt(targetDataX1Y1));
    }
    if(all(tempPos4 >= 0 && tempPos4 <= VolumeSizeMinusOne))
    {
        InterlockedAdd(WindBufferOutput[tempPos4.xyz], -PackFloatToInt(targetDataZ1));
    }
    if(all(tempPos5 >= 0 && tempPos5 <= VolumeSizeMinusOne))
    {
        InterlockedAdd(WindBufferOutput[tempPos5.xyz], -PackFloatToInt(targetDataX1Z1));
    }
    if(all(tempPos6 >= 0 && tempPos6 <= VolumeSizeMinusOne))
    {
        InterlockedAdd(WindBufferOutput[tempPos6.xyz], -PackFloatToInt(targetDataY1Z1));
    }
    if(all(tempPos7 >= 0 && tempPos7 <= VolumeSizeMinusOne))
    {
        InterlockedAdd(WindBufferOutput[tempPos7.xyz], -PackFloatToInt(targetDataX1Y1Z1));
    }
    if(all(dispatchThreadID <= VolumeSizeMinusOne))
    {
        float cellData = targetData + targetDataX1 + targetDataY1 + targetDataX1Y1
                        + targetDataZ1 + targetDataX1Z1 + targetDataY1Z1
                        + targetDataX1Y1Z1;
        InterlockedAdd(WindBufferOutput[dispatchThreadID.xyz], PackFloatToInt(cellData));
    }
}
The final wind information for each channel is stored in a single 3D RT
The logic of this part is relatively simple, just pass the corresponding RT into the ComputeShader for channel mixing

#pragma kernel CSMain

Texture3D<int> WindBufferInputX;
Texture3D<int> WindBufferInputY;
Texture3D<int> WindBufferInputZ;
RWTexture3D<float3> WindBufferOutput;

#define N 4

[numthreads(N,N,N)]
void CSMain (int3 dispatchThreadID : SV_DispatchThreadID)
{
    float x = PackIntToFloat(WindBufferInputX[dispatchThreadID.xyz].r);
    float y = PackIntToFloat(WindBufferInputY[dispatchThreadID.xyz].r);
    float z = PackIntToFloat(WindBufferInputZ[dispatchThreadID.xyz].r);
    WindBufferOutput[dispatchThreadID.xyz] = float3(x,y,z);
}
The running effect on the mobile phone
The phone uses its own Huawei Mate30 Pro

First of all, when publishing, you need to choose Vulcan (very important)

Because at the moment it seems that only the Vulcan platform supports ComputeShader

The overall rendering time is as shown in the figure below, and it seems to be acceptable (?)

The frame rate can't go up because of the GPUInstancing grass, I turned on this thing very high in order to see the effect, and the number of noodles is millions......

The grass is created with geometry shaders, which can refer to the warehouse of this boss

There are not many changes, that is, the wind force inside is replaced with the value of 3DRT, and the algorithm of the 100 million vertex animation is adjusted

https://github.com/wlgys8/URPLearn/tree/master/Assets/URPLearn/GrassGPUInstances
github.com/wlgys8/URPLearn/tree/master/Assets/URPLearn/GrassGPUInstances

Figure 11 Performance graph (trough)

Figure 12 Performance diagram (crest)
Finally, add 100 million things
The entire debugging UI is very ugly and not fully functional, but it is mainly to make it run on mobile phones

It is possible to manipulate the lens movement, create a Motor, move a Motor and debug some properties of the Motor, as well as view the RT output of the final wind information


I'll finish the final project and upload it to git, so stay tuned

(The first time I posted Zhihu, the content is not too technical, if there is something wrong to say, I hope you will point it out (compared to the heart.)

Project Repository: GitHub - SaberZG/GodOfWarWindSimulation: The wind field system of God of War 4 is implemented using Unity Compute Shader and ECS system replication

reference
Si Hu Hu: Wind Simulation in 'God of War' (GDC2019)

DirectX11 uses computational shaders to achieve Gaussian blur

https://github.com/wlgys8/URPLearn/tree/master/Assets/URPLearn/GrassGPUInstances

Edited on 2023-06-18 20:28・Guangdong