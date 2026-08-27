# تشغيل منصة INWP

هذا المجلد يقدّم تشغيلًا محليًا موحدًا لخدمة الحضور ومحرك المزامنة وقاعدة PostgreSQL وحافلة NATS. الهدف من هذا التشغيل هو اختبار المسار الكامل: تسجيل حدث حضور، حفظه في `attendance.clock_events`، وضعه في `sync.outbox`، نشره على NATS، ثم استهلاكه في محرك Rust وحفظه في `sync.event_store` وإدخاله إلى طابور المزامنة.

## البدء

انسخ الإعدادات النموذجية وعدّل كلمة مرور قاعدة البيانات قبل أي تشغيل مشترك:

```bash
cp deploy/.env.example deploy/.env
sudo docker compose --env-file deploy/.env -f deploy/docker-compose.full.yml up --build
```

تُنشأ مخططات PostgreSQL الخاصة بالحضور عند أول إنشاء لحجم قاعدة البيانات، بينما يشغّل محرك المزامنة migrations الخاصة به. عند استخدام حجم قاعدة بيانات موجود مسبقًا، طبّق ملف `attendance-service/migrations/002_add_outbox_last_error.up.sql` يدويًا أو عبر أداة migrations المعتمدة في بيئتك.

## نقاط الصحة

| الخدمة | العنوان | الغرض |
|---|---|---|
| خدمة الحضور HTTP | `http://localhost:8080/healthz` | فحص أن العملية تستجيب |
| خدمة الحضور readiness | `http://localhost:8080/readyz` | فحص جاهزية مسارات المستودع |
| محرك المزامنة | `http://localhost:8082/healthz` | فحص تشغيل محرك Rust |
| gRPC الحضور | `localhost:9090` | ClockIn وClockOut وGetEvents |
| gRPC المزامنة | `localhost:50052` | Merkle وGetDelta وApplyDelta والحالة |
| NATS | `localhost:4222` | نقل أحداث outbox |

## المسار الوظيفي الحالي

تستقبل خدمة الحضور طلب `clock-in` أو `clock-out` بعد التحقق من UUID والطابع الزمني، وتتحقق من التكرار ومن توفر التحقق البيومتري عند إرسال بيانات بيومترية. بعد الحفظ، يُنشأ payload JSON موحد في outbox. مرحّل NATS يطالب بالعناصر pending ويحوّلها إلى `SyncEvent`، ثم يرسلها إلى `inwp.sync.event.clock_event` ويضع الحالة `sent` أو `failed` مع إمكانية إعادة العناصر العالقة.

يستهلك محرك Rust الموضوع نفسه، ويتحقق من حجم الحدث وبنية JSON، ثم يحفظ الحدث في EventStore ويضيفه إلى QueueRepo. واجهات gRPC تعيد Merkle root والدلتا من PostgreSQL وتطبّق الدلتا الواردة وتحدّث نقطة التحقق، كما تعرض حالة التعافي والعقدة والأقران والنبضات.

## متطلبات الإنتاج

لا تستخدم القيم الموجودة في `.env.example` في بيئة حقيقية. يجب تفعيل mTLS لمحرك المزامنة بتوفير شهادة الخادم والمفتاح وشهادة CA وتمرير `SYNC_MTLS_REQUIRED=true`، كما يجب وضع خدمة الحضور خلف هوية وصلاحيات حقيقية بدل اعتبار readiness مصادقة. ينبغي تشغيل قاعدة البيانات بحساب تطبيق غير مالك للجداول مع تطبيق RLS عبر سياق الوزارة، واستخدام JetStream أو وسيط موثوق مع سياسة retention ومراقبة للرسائل.

وضع Compose الحالي يختار NATS على أنه ناقل الأحداث عند توفره، ويستخدم NoopPublisher عند غيابه. هذا مناسب للتطوير المحلي فقط؛ في الإنتاج يجب اعتبار غياب NATS خطأ نشر لا وضعًا صامتًا.

## الفحص المحلي

يمكن تنفيذ فحوصات الكود دون Docker:

```bash
cd attendance-service
 go test -race ./...
 go vet ./...
 go build ./...

cd ../backend/sync-engine
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --all-targets --all-features -- -D warnings
```
