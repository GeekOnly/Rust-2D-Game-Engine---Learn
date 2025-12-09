# Scene File Fix - แก้ไข main.json

## ปัญหา

```
[2025-12-09T16:43:45Z ERROR engine] Failed to load scene: 
invalid type: string "next_entity", expected u32 at line 605 column 15
```

## สาเหตุ

ไฟล์ `projects/Celeste Demo/scenes/main.json` มี **2 JSON objects ซ้อนกัน**:

```json
{
  "next_entity": 318,
  "transforms": [
    ...
  ],
  [
  "next_entity": 491,  // ← ตัวที่ 2 (ผิด!)
  "transforms": [
    ...
```

บรรทัด 605 มี `"next_entity": 491` ที่อยู่ผิดที่ ทำให้ JSON parser สับสน

## การแก้ไข

1. **สำรองไฟล์เดิม**
   ```
   main.json → main.json.backup
   ```

2. **ลบ JSON object ที่ 2 ออก**
   - ลบจากบรรทัด 605 ถึงจบไฟล์
   - เหลือแค่ JSON object แรก (next_entity: 318)

3. **เพิ่ม closing brackets**
   ```json
       ]
     ]
   }
   ```

## ผลลัพธ์

✅ ไฟล์ main.json มี structure ถูกต้องแล้ว
✅ มีแค่ 1 JSON object
✅ มี next_entity เดียว (318)
✅ JSON ปิดอย่างถูกต้อง

## ไฟล์ที่แก้ไข

- `projects/Celeste Demo/scenes/main.json` - แก้ไข JSON structure
- `projects/Celeste Demo/scenes/main.json.backup` - สำรองไฟล์เดิม
- `projects/Celeste Demo/scenes/main.scene` - ไม่มีปัญหา (ถูกต้องอยู่แล้ว)

## การทดสอบ

ลองรัน engine อีกครั้ง:
```bash
cargo run --release
```

Scene ควรโหลดได้โดยไม่มี error แล้ว!

## หมายเหตุ

- main.scene ไม่มีปัญหา (next_entity: 29)
- main.json เป็นไฟล์ที่มีปัญหา (มี 2 objects)
- อาจเกิดจากการ merge หรือ save ผิดพลาด
- ถ้ามีปัญหาสามารถ restore จาก main.json.backup ได้
