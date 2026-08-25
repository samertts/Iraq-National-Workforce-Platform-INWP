# تقرير تنفيذ وإكمال منصة INWP

**التاريخ:** 25 أغسطس 2026

**المستودع:** [Iraq-National-Workforce-Platform-INWP][1]

**فرع التنفيذ:** `feature/platform-completion-hardening`

**طلب الدمج:** [Pull Request #19][2]

## الملخص التنفيذي

تم تحويل الجزء الأكثر أهمية من المخطط إلى مسار تشغيلي فعلي قابل للبناء والاختبار: تسجيل حدث حضور، حفظه في PostgreSQL، وضع نسخة مزامنة منه في outbox، تمريره عبر NATS، استهلاكه في محرك Rust، حفظه في EventStore، ثم إدخاله في طابور المزامنة. هذا يختلف جوهريًا عن الحالة السابقة التي كانت تحتوي على واجهات ومعمارية واسعة لكن مع مستودعات تعيد كيانات فارغة، وواجهات gRPC غير مسجلة فعليًا، ومسار مزامنة يعيد ردودًا ثابتة أو `Not Implemented`.

النتيجة الحالية هي **نواة Backend عاملة لمسار الحضور والمزامنة** وليست بعدُ منصة قوى عاملة وطنية مكتملة من جميع الجوانب. أصبح المسار العمودي الأساسي قابلًا للتشغيل، بينما تبقى خدمات الهوية والموظفين والإجازات والرواتب والواجهة الأمامية وإدارة الصلاحيات المتقدمة مراحل مستقلة تحتاج إلى تنفيذ إضافي.

## ما تم إصلاحه وتنفيذه

| المجال | التنفيذ الحالي |
|---|---|
| خدمة الحضور | إصلاح إعادة بناء أحداث الحضور والورديات والسياسات والاستثناءات من PostgreSQL مع الحفاظ على الحقول الاختيارية والحالة والإصدار. |
| التحقق من المدخلات | إيقاف استخدام `uuid.MustParse` في مسارات REST، وإرجاع Problem Details للأخطاء، والتحقق من الوقت وUUIDs ومدى التواريخ. |
| الحضور البيومتري | عند إرسال بيانات بيومترية دون موفر تحقق موصول، تعيد الخدمة خطأً قابلًا للمعالجة بدل panic أو قبول تحقق وهمي. |
| outbox | حفظ payload الكامل، والمطالبة الذرية بالعناصر عبر `UPDATE ... FOR UPDATE SKIP LOCKED ... RETURNING`، ودعم حالات `processing` و`sent` و`failed` وإعادة ضبط العناصر العالقة. |
| gRPC الحضور | إضافة عقد protobuf مولد وRegister فعلي لخدمة ClockIn وClockOut وGetEvents. |
| GULA | توصيل الناشر اختياريًا عند توفر `GULA_BASE_URL` و`GULA_ACCESS_TOKEN`. |
| NATS relay | إضافة مرحّل يحوّل outbox إلى عقد SyncEvent موحد ويرسله إلى `inwp.sync.event.clock_event` مع مهلة flush. |
| محرك Rust | تنفيذ GetMerkleRoot وGetDelta وApplyDelta ومسار replay الأساسي، وإرجاع بيانات حقيقية من EventStore بدل الردود الفارغة. |
| جسر Rust/NATS | استهلاك أحداث الحضور، التحقق من الحجم والبنية، حفظها في EventStore، ثم إدخالها في QueueRepo. |
| تسجيل العقدة | تسجيل العقدة المحلية في `sync.node_registry` قبل إدخال الأحداث إلى queue، مع قدرات العقدة وحالتها ونطاقها. |
| نقاط التحقق | تحديث Merkle checkpoint بعد ApplyDelta. |
| التعافي | عرض RecoveryStateMachine عبر gRPC في status وGetRecoveryState. |
| TLS/mTLS | تفعيل TLS وmTLS في خادم Rust عند وجود الشهادات، مع رفض التشغيل عندما يكون mTLS مطلوبًا والشهادات مفقودة. النمط غير المشفر متاح للتطوير فقط عند تعطيل mTLS صراحة. |
| الرصد والصحة | إضافة `/healthz` و`/readyz` لخدمة الحضور وخادم صحة لمحرك Rust على المنفذ المعلن. |
| التشغيل | إضافة `deploy/docker-compose.full.yml` لتشغيل PostgreSQL وNATS وخدمة الحضور ومحرك المزامنة مع migrations وhealthchecks. |
| CI | إضافة فحص Rust format/check/test/clippy، تثبيت protobuf-compiler، والتحقق من Compose. |
| الاختبارات | إضافة اختبارات clock-in لمسار payload والتكرار والوقت المستقبلي وغياب البيومترية. |

## المسار التشغيلي الذي أصبح يعمل

يبدأ العميل بطلب `POST /api/v1/attendance/clock-in`. تتحقق الخدمة من المدخلات، وتكشف التكرار، ثم تحفظ الحدث في `attendance.clock_events` وتنشئ payload موحدًا داخل `sync.outbox`. مرحّل NATS يطالب بالصفوف pending، ويحوّل payload إلى حدث مزامنة JSON، ثم ينشره على الموضوع `inwp.sync.event.clock_event`.

يستقبل محرك Rust الحدث عبر جسر NATS. بعد التحقق من حجم payload وصحة البنية، يحفظ الحدث في `sync.event_store` ويسجله في `sync.sync_queue` بأولوية صحيحة، مع ضمان وجود العقدة في `sync.node_registry`. بعد ذلك تستطيع واجهات Rust إعادة Merkle root والدلتا، وتطبيق الدلتا الواردة وتحديث checkpoint.

## نتائج التحقق الفعلي

| الفحص | النتيجة |
|---|---|
| `go test -race ./...` | ناجح |
| `go vet ./...` | ناجح |
| `go build ./...` | ناجح |
| `cargo fmt --check` | ناجح |
| `cargo check --locked` | ناجح |
| `cargo test --locked` | ناجح؛ 15 اختبارًا |
| `cargo clippy --all-targets --all-features -- -D warnings` | ناجح |
| `docker compose ... config --quiet` | ناجح |
| بناء صورة خدمة الحضور | ناجح |
| بناء صورة محرك المزامنة | ناجح |
| GitHub Actions | ناجح بجميع الوظائف في التشغيل الأخير؛ [Run 32831002238][3] |

تم تنفيذ smoke test حي باستخدام PostgreSQL وNATS وصور Docker. بعد إرسال clock-in جديد كانت النتائج: حدثان بحالة `sent` في outbox، حدثان في `sync.event_store`، وعنصر واحد في `sync.sync_queue` بحالة `pending` وأولوية `5`. كما استجابت نقاط الصحة لخدمة الحضور ومحرك المزامنة بنجاح.

## طريقة التشغيل

```bash
cd Iraq-National-Workforce-Platform-INWP
cp deploy/.env.example deploy/.env
# غيّر POSTGRES_PASSWORD قبل التشغيل الحقيقي
sudo docker compose --env-file deploy/.env -f deploy/docker-compose.full.yml up --build
```

للتفاصيل التشغيلية ونقاط الصحة ومتطلبات الإنتاج، راجع [deploy/README.ar.md](deploy/README.ar.md). مسارات الصحة الأساسية هي `http://localhost:8080/healthz` لخدمة الحضور، و`http://localhost:8080/readyz` لجاهزيتها، و`http://localhost:8082/healthz` لمحرك المزامنة.

## ما لم يكتمل بعد

هذا الإصلاح لا يدّعي أن المنصة الوطنية الكاملة اكتملت. ما يزال تنفيذ هوية المستخدمين والمصادقة والتفويض متعدد المستويات، وسجل الموظفين المركزي، وخدمات الوزارات والمواقع، والإجازات والدوام الرسمي والرواتب، ولوحات المعلومات والتقارير، وواجهة الويب أو تطبيق الأجهزة، وإدارة الأجهزة البيومترية الفعلية، وربط دليل الموظفين الوطني، واختبارات المزامنة بين عقد متعددة، والتشغيل الإنتاجي عالي التوافر، أعمالًا لاحقة.

كما أن وضع Compose التطويري يعطل mTLS صراحة لأن شهادات التطوير غير موجودة. قبل الإنتاج يجب توفير شهادات حقيقية، تفعيل `SYNC_MTLS_REQUIRED=true`، استخدام حساب قاعدة بيانات لا يملك الجداول، تمرير سياق الوزارة إلى RLS بطريقة آمنة، وإضافة مصادقة فعلية لكل واجهات HTTP وgRPC. وينبغي كذلك تحويل غياب NATS في الإنتاج إلى خطأ نشر بدل الرجوع الصامت إلى NoopPublisher.

تعذر تشغيل `docker compose up` بالشبكة الافتراضية داخل بيئة الاختبار بسبب قيد kernel/iptables على جدول `raw`، وليس بسبب خطأ في إعداد Compose؛ تم تجاوز ذلك ببناء الصور واختبار الحاويات عبر `host network`، كما مرّ فحص Compose في GitHub Actions بنجاح.

## سجل التسليم

| العنصر | القيمة |
|---|---|
| Commit الوظائف الأساسية | `aee1051` |
| Commit إصلاح CI | `0eac320` |
| الفرع | `feature/platform-completion-hardening` |
| Pull Request | `#19` |
| حالة CI الأخيرة | ناجحة |

> التقييم المهني: أصبحت المنصة الآن **قابلة للبناء والتشغيل لمسار الحضور والمزامنة الأساسي**، لكنها لا تزال **نسخة Backend متقدمة وعمودية** وليست منتجًا وطنيًا مكتملًا حتى تُنفّذ طبقات الهوية والبيانات الوظيفية والواجهة والتشغيل الإنتاجي.

## المراجع

[1]: https://github.com/samertts/Iraq-National-Workforce-Platform-INWP "مستودع INWP على GitHub"

[2]: https://github.com/samertts/Iraq-National-Workforce-Platform-INWP/pull/19 "Pull Request #19"

[3]: https://github.com/samertts/Iraq-National-Workforce-Platform-INWP/actions/runs/32831002238 "تشغيل CI الأخير"
