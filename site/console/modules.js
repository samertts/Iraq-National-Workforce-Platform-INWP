/* Static, synthetic feature gallery. No network, storage, or administrative decisions. */
(() => {
  const groups = {
    ar: {
      service: 'الخدمات والتواصل', people: 'الموظفون والتطوير', planning: 'التخطيط والضمان',
      insight: 'الرؤى والتكامل', governance: 'الحوكمة والخصوصية', extras: 'خدمات مساندة'
    },
    en: {
      service: 'Services & communication', people: 'People & development', planning: 'Planning & assurance',
      insight: 'Insights & integrations', governance: 'Governance & privacy', extras: 'Additional services'
    }
  };
  const messages = {
    ar: {
      galleryLabel: 'معرض وحدات المنصة · محاكاة محلية',
      intro: 'استكشف التصورات الوظيفية. التفاعلات مؤقتة داخل الصفحة، ولا تُرسل أو تحفظ بيانات.',
      scope: 'تصور توضيحي فقط', notConnected: 'غير متصل بالخدمات', open: 'استعراض النموذج', back: 'العودة إلى المعرض',
      limitation: 'حدود هذه المعاينة', synthetic: 'مثال اصطناعي', demoAction: 'محاكاة الإجراء', simulated: 'تمت المحاكاة محليًا فقط؛ لم يُحفظ أو يُرسل أي شيء.',
      confirm: 'تأكيد المعاينة', selected: 'الخيار المحدد', choose: 'اختر موضوعًا', sampleFlow: 'مسار توضيحي', sampleState: 'حالة اصطناعية', noRealDecision: 'لا ينتج عن هذه المحاكاة قرار إداري أو اعتماد رسمي.',
      transactionsTitle:'مركز المعاملات', transactionsDesc:'متابعة الطلبات الإدارية والنواقص والموافقات بحسب الدور.', transactionsLimit:'يتطلب تعريف أنواع المعاملات ومصفوفة الاعتماد الرسمية قبل التشغيل.',
      announcementsTitle:'التعاميم والسياسات', announcementsDesc:'نشر التعاميم حسب الجهة والصفة، مع سجل نسخ وإثبات اطلاع.', announcementsLimit:'لا يحتوي هذا العرض على تعميم رسمي أو إثبات اطلاع حقيقي.',
      ticketsTitle:'تذاكر الدعم', ticketsDesc:'توجيه الاستفسارات إلى الفريق المختص ومتابعة مراحل المعالجة.', ticketsLimit:'لا تكتب وصف حالة حقيقية؛ نموذج المعاينة لا يرسل ولا يحتفظ بالبيانات.',
      delegationTitle:'التفويض المؤقت', delegationDesc:'تفويض مهمة محدودة بمدة ونطاق، مع انتهاء ومراجعة تلقائيين.', delegationLimit:'لا يمنح هذا العرض أي صلاحية أو تفويض فعلي.',
      calendarTitle:'التقويم الإداري', calendarDesc:'عرض مواعيد الإجراءات والتكليفات والتنبيهات حسب الصلاحية.', calendarLimit:'المواعيد المعروضة رمزية ولا تمثل مواعيد حكومية أو قانونية.',
      skillsTitle:'المهارات والتدريب', skillsDesc:'دليل مهارات ودورات مع تسجيل التقدم ومصدر التحقق.', skillsLimit:'لا توجد ملفات مهارات أو تسجيلات تدريب فعلية.',
      opportunitiesTitle:'الفرص الداخلية', opportunitiesDesc:'عرض فرص التكليف والتطوير والتقديم عليها وفق شروط معلنة.', opportunitiesLimit:'لا توجد ترشيحات أو قرارات أهلية أو ترتيب آلي للأشخاص.',
      handoverTitle:'نقل المعرفة والتسليم', handoverDesc:'قوائم تسليم للمهام والوثائق والمعرفة عند تبدل المسؤولية.', handoverLimit:'قائمة العرض مؤقتة ولا تمثل محضر تسليم محفوظًا.',
      planningTitle:'محاكي تخطيط القوى العاملة', planningDesc:'استكشاف سيناريو افتراضي لتغير الاحتياج والمهارات.', planningLimit:'حساب توضيحي ثابت الافتراضات؛ ليس تنبؤًا أو توصية تخطيطية.',
      qualityTitle:'جودة البيانات', qualityDesc:'رصد النقص والتكرار والتعارض وإحالة التصحيحات للمراجعة.', qualityLimit:'المخالفات اصطناعية ولا يجري فحص سجل فعلي أو تعديل بيانات.',
      dashboardTitle:'لوحة المؤشرات المجمعة', dashboardDesc:'مؤشرات إجمالية تساعد التخطيط مع تقليل بيانات الأفراد.', dashboardLimit:'الأرقام أمثلة غير رسمية ولا تمثل إحصاءات العراق أو جهة بعينها.',
      integrationsTitle:'دليل التكاملات', integrationsDesc:'كتالوج عقود الأنظمة ومصادر البيانات وحالة توافقها.', integrationsLimit:'لا يوجد اتصال أو فحص حي أو عناوين أو مفاتيح تكامل.',
      continuityTitle:'استمرارية العمل', continuityDesc:'خطط المهام الحرجة والبدائل وإجراءات استعادة الخدمة.', continuityLimit:'لا توجد جهات اتصال حقيقية أو خطة طوارئ معتمدة في هذا العرض.',
      publicDataTitle:'بوابة المؤشرات العامة', publicDataDesc:'نشر بيانات تجميعية بعد اعتمادها وفحص مخاطر كشف الهوية.', publicDataLimit:'التنزيل التجريبي يحتوي أرقامًا مصطنعة فقط، وليس بيانات رسمية.',
      suggestionsTitle:'الاقتراحات والتحسين', suggestionsDesc:'استقبال أفكار التحسين وتصنيفها ومتابعة دراستها.', suggestionsLimit:'لا تدخل نصًا حرًا أو معلومات شخصية؛ الاختيار لا يُرسل ولا يُحفظ.',
      guideTitle:'الدليل الإجرائي', guideDesc:'إرشاد المستخدم إلى خطوات الخدمة ومصادرها المعتمدة.', guideLimit:'إجابات ثابتة توضيحية وليست استشارة قانونية أو قرارًا إداريًا.',
      notificationsTitle:'مركز التنبيهات', notificationsDesc:'تنظيم إشعارات المهام والمواعيد حسب تفضيلات المستخدم.', notificationsLimit:'التنبيهات أمثلة ولا ترتبط بحساب أو قناة اتصال.',
      onboardingTitle:'تهيئة الموظف الجديد', onboardingDesc:'قائمة تعريف بالخدمات والسياسات والمهام الأساسية.', onboardingLimit:'لا تُنشأ حسابات أو صلاحيات أو سجلات موظفين من هذه المعاينة.',
      privacyTitle:'مراجعة الخصوصية وطلبات البيانات', privacyDesc:'مسار مقترح لمراجعة الوصول والتصحيح والاحتفاظ بالبيانات.', privacyLimit:'ليس إجراءً قانونيًا نافذًا؛ يلزم اعتماد السياسة والجهة المختصة.',
      dashboardPrefsTitle:'لوحة قابلة للتخصيص', dashboardPrefsDesc:'ترتيب بطاقات المؤشرات حسب الدور واحتياج العمل.', dashboardPrefsLimit:'تغييرات العرض مؤقتة لهذه الجلسة ولا تُحفظ.',
      queueName:'قائمة افتراضية', stage:'المرحلة', sampleDate:'موعد رمزي', inReview:'قيد المراجعة (نموذج)',
      ack:'تأكيد الاطلاع (محاكاة)', createTicket:'إنشاء تذكرة تجريبية', ticketCategory:'فئة الطلب', general:'استفسار عام', accessHelp:'مساعدة في الوصول', processHelp:'استفسار عن إجراء',
      scenarioSize:'حجم السيناريو الافتراضي', scenarioValue:'وحدات افتراضية', scenarioExplain:'الناتج توضيحي وفق افتراض ثابت داخل المتصفح فقط.',
      qaWarnings:'ملاحظات في عينة مصطنعة', qualityRows:['حقل مطلوب غير مكتمل (DEMO)','رمز وحدة مكرر (DEMO)','تاريخ متعارض (DEMO)'],
      interested:'إبداء اهتمام (محاكاة)', applied:'تم تسجيل اهتمام تجريبي محليًا.', course:'مقدمة في إدارة البيانات', opportunity:'مشروع توثيق إجراء افتراضي',
      checklist:'قائمة تسليم تجريبية', itemOne:'تحديد المهام المفتوحة', itemTwo:'حصر الوثائق الإجرائية', itemThree:'تحديد جهة اتصال بديلة',
      continuityStep:'تمرين استمرارية افتراضي', stepOne:'تحديد خدمة ذات أولوية', stepTwo:'تحديد بديل افتراضي', stepThree:'مراجعة نسخة الاستعادة',
      announcement:'تعميم تجريبي DEMO-02', announcementBody:'مثال على عرض سياسة داخلية بعد مراجعتها واعتمادها من الجهة المختصة.',
      questions:['كيف أتابع معاملة؟','ما الذي أحتاجه قبل تقديم طلب؟','أين أتحقق من سياسة رسمية؟'],
      answers:['افتح مركز المعاملات واختر رقم الطلب. في النظام الحقيقي تظهر الخطوات المسموح بها حسب صلاحيتك.','راجع قائمة المتطلبات في صفحة الخدمة قبل البدء؛ لا ترفق معلومات حساسة إلا عبر القناة المعتمدة.','ارجع إلى مصدر الجهة الرسمي أو مسؤولها المخول؛ هذا الدليل لا ينشئ قواعد رسمية.'],
      suggestionChoices:['تحسين سهولة الاستخدام','توضيح متطلبات خدمة','اقتراح تدريب'], submitSuggestion:'عرض تأكيد تجريبي',
      exportSample:'تنزيل عينة اصطناعية', exportDone:'تم تجهيز ملف تجريبي محلي للتنزيل.',
      notifications:['تذكير تجريبي: مراجعة طلب افتراضي','إعلان نموذجي: تحديث دليل الخدمة','تنبيه تجريبي: موعد رمزي قريب'],
      onboarding:['التعرف على الخدمات','قراءة السياسات المعتمدة','التواصل مع نقطة الدعم'],
      privacyFlow:['استلام الطلب من قناة موثوقة','التحقق من هوية وصلاحية صاحب الطلب','مراجعة الأثر والموافقة قبل التنفيذ'],
      prefHint:'هذه الصفحة لا تحفظ ترتيب البطاقات؛ التخصيص الحقيقي يحتاج ملف تفضيلات محميًا وصلاحيات مناسبة.',
      categoryService:'الخدمات والتواصل', categoryPeople:'الموظفون والتطوير', categoryPlanning:'التخطيط والضمان', categoryInsight:'الرؤى والتكامل', categoryGovernance:'الحوكمة والخصوصية', categoryExtras:'خدمات مساندة'
    },
    en: {
      galleryLabel: 'Platform module gallery · local simulation',
      intro: 'Explore functional concepts. Interactions are temporary in this page and do not submit or store data.',
      scope: 'Illustrative concept only', notConnected: 'Not connected to services', open: 'Explore demo', back: 'Back to gallery',
      limitation: 'Preview limitations', synthetic: 'Synthetic example', demoAction: 'Simulate action', simulated: 'Simulated locally only; nothing was saved or sent.',
      confirm: 'Confirm preview', selected: 'Selected option', choose: 'Choose a topic', sampleFlow: 'Illustrative flow', sampleState: 'Synthetic state', noRealDecision: 'This simulation does not make an administrative decision or official approval.',
      transactionsTitle:'Transaction center', transactionsDesc:'Track administrative requests, missing items, and approvals by role.', transactionsLimit:'Official transaction types and approval matrix must be defined before operation.',
      announcementsTitle:'Announcements & policies', announcementsDesc:'Publish notices by organization and role, with version history and acknowledgements.', announcementsLimit:'This preview contains no official notice or real acknowledgement.',
      ticketsTitle:'Support tickets', ticketsDesc:'Route questions to the appropriate team and track handling stages.', ticketsLimit:'Do not enter a real case description; this preview does not submit or retain data.',
      delegationTitle:'Temporary delegation', delegationDesc:'Delegate a limited task for a defined period and scope, with expiry and review.', delegationLimit:'This preview grants no permissions or delegation.',
      calendarTitle:'Administrative calendar', calendarDesc:'Show process, assignment, and reminder dates according to access scope.', calendarLimit:'Displayed dates are symbolic, not government or legal deadlines.',
      skillsTitle:'Skills & learning', skillsDesc:'Skills directory and courses with progress and verification sources.', skillsLimit:'No actual skill profiles or course registrations are present.',
      opportunitiesTitle:'Internal opportunities', opportunitiesDesc:'List assignment and development opportunities with published criteria.', opportunitiesLimit:'There is no ranking, eligibility decision, or person recommendation.',
      handoverTitle:'Knowledge handover', handoverDesc:'Handover lists for tasks, documents, and knowledge when responsibilities change.', handoverLimit:'The list is temporary and is not a saved handover record.',
      planningTitle:'Workforce planning simulator', planningDesc:'Explore a fictional scenario for changing capacity and skills needs.', planningLimit:'A fixed illustrative calculation, not a forecast or planning recommendation.',
      qualityTitle:'Data quality', qualityDesc:'Identify missing, duplicate, or conflicting data and route corrections for review.', qualityLimit:'Issues are synthetic; no real records are checked or changed.',
      dashboardTitle:'Aggregated dashboard', dashboardDesc:'Aggregate indicators to support planning while minimizing person-level data.', dashboardLimit:'Figures are fictional examples, not official Iraqi or agency statistics.',
      integrationsTitle:'Integration directory', integrationsDesc:'Catalog system contracts, data sources, and compatibility status.', integrationsLimit:'No live connection, health check, endpoint, or integration key is present.',
      continuityTitle:'Business continuity', continuityDesc:'Plans for critical tasks, backups, and service recovery.', continuityLimit:'No real contacts or approved emergency plan are included.',
      publicDataTitle:'Public indicators portal', publicDataDesc:'Publish aggregate data after approval and disclosure-risk review.', publicDataLimit:'The sample download contains synthetic numbers only, not official data.',
      suggestionsTitle:'Suggestions & improvement', suggestionsDesc:'Collect, classify, and track improvement ideas.', suggestionsLimit:'Do not enter free text or personal information; choices are not sent or stored.',
      guideTitle:'Process guidance', guideDesc:'Guide users through service steps and approved sources.', guideLimit:'Static illustrative answers, not legal advice or administrative decisions.',
      notificationsTitle:'Notification center', notificationsDesc:'Organize task and date reminders by user preference.', notificationsLimit:'Examples are not linked to an account or delivery channel.',
      onboardingTitle:'New employee onboarding', onboardingDesc:'An introductory checklist for services, policies, and essential tasks.', onboardingLimit:'This preview creates no account, permission, or employee record.',
      privacyTitle:'Privacy review & data requests', privacyDesc:'A proposed route to review access, correction, and retention requests.', privacyLimit:'Not an operative legal process; policy and competent authority require approval.',
      dashboardPrefsTitle:'Configurable dashboard', dashboardPrefsDesc:'Arrange indicator cards by role and work needs.', dashboardPrefsLimit:'Display changes are temporary and are not saved.',
      queueName:'Fictional queue', stage:'Stage', sampleDate:'Sample date', inReview:'Under review (sample)',
      ack:'Acknowledge (simulation)', createTicket:'Create sample ticket', ticketCategory:'Request category', general:'General question', accessHelp:'Access support', processHelp:'Process question',
      scenarioSize:'Fictional scenario size', scenarioValue:'Sample units', scenarioExplain:'Illustrative output from a fixed assumption in this browser only.',
      qaWarnings:'Notes in a synthetic sample', qualityRows:['Required field incomplete (DEMO)','Duplicate unit code (DEMO)','Conflicting date (DEMO)'],
      interested:'Register interest (simulation)', applied:'Sample interest recorded locally.', course:'Introduction to data stewardship', opportunity:'Fictional process documentation project',
      checklist:'Sample handover checklist', itemOne:'Identify open tasks', itemTwo:'List process documents', itemThree:'Identify alternate contact',
      continuityStep:'Sample continuity exercise', stepOne:'Identify a priority service', stepTwo:'Name a fictional alternate', stepThree:'Review recovery copy',
      announcement:'Sample notice DEMO-02', announcementBody:'Example of displaying a policy after review and approval by the competent organization.',
      questions:['How do I track a request?','What should I prepare before applying?','Where can I verify an official policy?'],
      answers:['Open the transaction center and choose the request number. In an operational system, permitted steps depend on your authorization.','Review the service requirements before starting; submit sensitive information only through the approved channel.','Refer to the organization’s official source or authorized contact; this guide does not create official rules.'],
      suggestionChoices:['Improve usability','Clarify service requirements','Suggest training'], submitSuggestion:'Show sample confirmation',
      exportSample:'Download synthetic sample', exportDone:'A local synthetic sample file is ready to download.',
      notifications:['Sample reminder: review a fictional request','Sample notice: service guide update','Sample alert: a symbolic date is approaching'],
      onboarding:['Explore available services','Read approved policies','Contact the support point'],
      privacyFlow:['Receive the request through a trusted channel','Verify requester identity and authority','Review impact and obtain approval before action'],
      prefHint:'This page does not save card ordering; real personalization needs protected preferences and authorization.',
      categoryService:'Services & communication', categoryPeople:'People & development', categoryPlanning:'Planning & assurance', categoryInsight:'Insights & integrations', categoryGovernance:'Governance & privacy', categoryExtras:'Additional services'
    }
  };

  const features = [
    {id:'transactions',group:'service',title:'transactionsTitle',desc:'transactionsDesc',limit:'transactionsLimit',icon:'▤',kind:'queue'},
    {id:'announcements',group:'service',title:'announcementsTitle',desc:'announcementsDesc',limit:'announcementsLimit',icon:'◉',kind:'announcement'},
    {id:'tickets',group:'service',title:'ticketsTitle',desc:'ticketsDesc',limit:'ticketsLimit',icon:'◇',kind:'ticket'},
    {id:'suggestions',group:'service',title:'suggestionsTitle',desc:'suggestionsDesc',limit:'suggestionsLimit',icon:'✧',kind:'suggestion'},
    {id:'guide',group:'service',title:'guideTitle',desc:'guideDesc',limit:'guideLimit',icon:'?',kind:'guide'},
    {id:'delegation',group:'people',title:'delegationTitle',desc:'delegationDesc',limit:'delegationLimit',icon:'⇄',kind:'action'},
    {id:'calendar',group:'people',title:'calendarTitle',desc:'calendarDesc',limit:'calendarLimit',icon:'▦',kind:'calendar'},
    {id:'skills',group:'people',title:'skillsTitle',desc:'skillsDesc',limit:'skillsLimit',icon:'⌑',kind:'course'},
    {id:'opportunities',group:'people',title:'opportunitiesTitle',desc:'opportunitiesDesc',limit:'opportunitiesLimit',icon:'↗',kind:'opportunity'},
    {id:'handover',group:'people',title:'handoverTitle',desc:'handoverDesc',limit:'handoverLimit',icon:'⇢',kind:'checklist'},
    {id:'planning',group:'planning',title:'planningTitle',desc:'planningDesc',limit:'planningLimit',icon:'⌁',kind:'scenario'},
    {id:'quality',group:'planning',title:'qualityTitle',desc:'qualityDesc',limit:'qualityLimit',icon:'✓',kind:'quality'},
    {id:'continuity',group:'planning',title:'continuityTitle',desc:'continuityDesc',limit:'continuityLimit',icon:'⟳',kind:'continuity'},
    {id:'dashboard',group:'insight',title:'dashboardTitle',desc:'dashboardDesc',limit:'dashboardLimit',icon:'▥',kind:'dashboard'},
    {id:'integrations',group:'insight',title:'integrationsTitle',desc:'integrationsDesc',limit:'integrationsLimit',icon:'⌘',kind:'integrations'},
    {id:'publicData',group:'insight',title:'publicDataTitle',desc:'publicDataDesc',limit:'publicDataLimit',icon:'◫',kind:'export'},
    {id:'privacy',group:'governance',title:'privacyTitle',desc:'privacyDesc',limit:'privacyLimit',icon:'⛨',kind:'privacy'},
    {id:'notifications',group:'extras',title:'notificationsTitle',desc:'notificationsDesc',limit:'notificationsLimit',icon:'◌',kind:'notifications'},
    {id:'onboarding',group:'extras',title:'onboardingTitle',desc:'onboardingDesc',limit:'onboardingLimit',icon:'＋',kind:'onboarding'},
    {id:'dashboardPrefs',group:'extras',title:'dashboardPrefsTitle',desc:'dashboardPrefsDesc',limit:'prefHint',icon:'⚙',kind:'preferences'}
  ];

  let activeFeature = null;
  const volatileState = Object.create(null);
  const text = (lang, key) => messages[lang]?.[key] ?? key;
  function el(tag, className, value) {
    const element = document.createElement(tag);
    if (className) element.className = className;
    if (value !== undefined) element.textContent = String(value);
    return element;
  }
  function button(label, action, data = {}) {
    const item = el('button', 'button button-small module-action', label);
    item.type = 'button';
    item.dataset.moduleAction = action;
    for (const [key, value] of Object.entries(data)) item.dataset[key] = value;
    return item;
  }
  function note(lang, title, message) {
    const box = el('div', 'notice-card module-note');
    box.append(el('span', 'notice-icon', 'i'));
    const body = el('div');
    body.append(el('h3', '', title), el('p', '', message));
    box.append(body);
    return box;
  }
  function renderGallery(lang) {
    const root = el('div', 'view module-gallery');
    root.append(note(lang, text(lang, 'galleryLabel'), text(lang, 'intro')));
    const order = ['service','people','planning','insight','governance','extras'];
    for (const group of order) {
      const items = features.filter(feature => feature.group === group);
      const section = el('section', 'module-group');
      section.append(el('h2', 'module-group-title', text(lang, `category${group[0].toUpperCase()}${group.slice(1)}`)));
      const grid = el('div', 'module-grid');
      for (const feature of items) {
        const card = el('article', 'module-card');
        const head = el('div', 'module-card-head');
        head.append(el('span', 'module-icon', feature.icon));
        const badges = el('div', 'module-badges');
        badges.append(el('span', 'pill waiting', text(lang, 'scope')));
        card.append(head, el('h3', '', text(lang, feature.title)), el('p', '', text(lang, feature.desc)), badges);
        const open = button(text(lang, 'open'), 'open', {feature:feature.id});
        open.setAttribute('aria-label', `${text(lang, 'open')}: ${text(lang, feature.title)}`);
        card.append(open);
        grid.append(card);
      }
      section.append(grid);
      root.append(section);
    }
    root.append(note(lang, text(lang, 'notConnected'), text(lang, 'noRealDecision')));
    return root;
  }
  function addChecklist(parent, lang, feature, items) {
    const list = el('div', 'module-checklist');
    items.forEach((key, index) => {
      const label = el('label', 'module-check');
      const input = el('input');
      input.type = 'checkbox';
      input.checked = Boolean(volatileState[`${feature.id}:${index}`]);
      input.addEventListener('change', () => { volatileState[`${feature.id}:${index}`] = input.checked; });
      label.append(input, el('span', '', key));
      list.append(label);
    });
    parent.append(list);
  }
  function renderDetail(lang, feature) {
    const root = el('div', 'view module-detail');
    root.append(button(text(lang, 'back'), 'back'));
    const card = el('section', 'panel module-detail-card');
    const header = el('div', 'module-detail-head');
    header.append(el('span', 'module-icon', feature.icon));
    const titles = el('div');
    titles.append(el('span', 'pill waiting', text(lang, 'scope')), el('h2', '', text(lang, feature.title)), el('p', '', text(lang, feature.desc)));
    header.append(titles);
    card.append(header, note(lang, text(lang, 'limitation'), text(lang, feature.limit)));
    const samples = el('section', 'module-sample');
    samples.append(el('h3', '', text(lang, 'sampleFlow')));
    switch (feature.kind) {
      case 'queue':
        samples.append(el('p', '', `${text(lang, 'queueName')}: FLOW-DEMO-01 · ${text(lang, 'stage')}: ${text(lang, 'inReview')}`));
        samples.append(el('p', '', `FLOW-DEMO-02 · ${text(lang, 'stage')}: ${text(lang, 'sampleState')}`));
        break;
      case 'announcement': {
        const notice = el('article', 'sample-announcement');
        notice.append(el('strong', '', text(lang, 'announcement')), el('p', '', text(lang, 'announcementBody')));
        samples.append(notice, button(volatileState.ack ? text(lang, 'simulated') : text(lang, 'ack'), 'ack'));
        break;
      }
      case 'ticket': {
        const label = el('label', 'module-field', text(lang, 'ticketCategory'));
        const select = el('select');
        select.dataset.ticketSelect = 'true';
        for (const [value, key] of [['general','general'],['access','accessHelp'],['process','processHelp']]) {
          const option = el('option', '', text(lang, key)); option.value = value; select.append(option);
        }
        label.append(select);
        samples.append(label, button(text(lang, 'createTicket'), 'ticket'));
        if (volatileState.ticketCount) samples.append(el('p', 'module-result', `${text(lang, 'sampleState')}: TICKET-DEMO-${String(volatileState.ticketCount).padStart(2,'0')} · ${text(lang, volatileState.ticketCategory || 'general')}`));
        break;
      }
      case 'suggestion': {
        const label = el('label', 'module-field', text(lang, 'selected'));
        const select = el('select'); select.dataset.suggestionSelect = 'true';
        text(lang, 'suggestionChoices').forEach((value, index) => { const option = el('option','',value); option.value = String(index); select.append(option); });
        label.append(select);
        samples.append(label, button(text(lang, 'submitSuggestion'), 'suggestion'));
        break;
      }
      case 'guide':
        text(lang, 'questions').forEach((question, index) => samples.append(button(question, 'question', {question:String(index)})));
        if (volatileState.answer !== undefined) samples.append(note(lang, text(lang, 'sampleState'), text(lang, 'answers')[volatileState.answer]));
        break;
      case 'scenario': {
        const label = el('label', 'module-field', text(lang, 'scenarioSize'));
        const range = el('input'); range.type = 'range'; range.min = '1'; range.max = '10'; range.value = String(volatileState.scenario || 4); range.dataset.scenario = 'true';
        const output = el('output', 'module-result', `${range.value} ${text(lang, 'scenarioValue')}`);
        range.addEventListener('input', () => { volatileState.scenario = Number(range.value); output.textContent = `${range.value} ${text(lang, 'scenarioValue')} · ${Number(range.value) * 3} ${text(lang, 'scenarioValue')}`; });
        label.append(range, output);
        samples.append(label, el('p', '', text(lang, 'scenarioExplain')));
        break;
      }
      case 'quality':
        samples.append(el('p', '', text(lang, 'qaWarnings')));
        text(lang, 'qualityRows').forEach(item => samples.append(el('p', 'quality-warning', `• ${item}`)));
        break;
      case 'dashboard':
        samples.append(el('div', 'module-kpis'));
        ['12','7','3'].forEach((value, index) => {
          const item = el('div', 'module-mini-kpi'); item.append(el('strong','',value),el('small','',`${text(lang,'synthetic')} ${index+1}`)); samples.querySelector('.module-kpis').append(item);
        });
        break;
      case 'integrations':
        samples.append(el('p', '', 'SYSTEM-DEMO-A  →  CONTRACT-DEMO-01  →  SYSTEM-DEMO-B'));
        samples.append(button(text(lang, 'demoAction'), 'simulate'));
        break;
      case 'calendar':
        ['A · '+text(lang,'sampleDate'), 'B · '+text(lang,'sampleDate'), 'C · '+text(lang,'sampleDate')].forEach(item => samples.append(el('p','',item)));
        break;
      case 'course':
        samples.append(el('p','',text(lang,'course')), button(text(lang,'interested'),'interest'));
        break;
      case 'opportunity':
        samples.append(el('p','',text(lang,'opportunity')),button(text(lang,'interested'),'interest'));
        break;
      case 'checklist':
        addChecklist(samples, lang, feature, ['itemOne','itemTwo','itemThree'].map(key=>text(lang,key)));
        break;
      case 'continuity':
        samples.append(el('p','',text(lang,'continuityStep')));
        addChecklist(samples, lang, feature, ['stepOne','stepTwo','stepThree'].map(key=>text(lang,key)));
        break;
      case 'export':
        samples.append(el('p','',`${text(lang,'synthetic')}: CATEGORY-A 12 · CATEGORY-B 7 · CATEGORY-C 3`),button(text(lang,'exportSample'),'export'));
        break;
      case 'privacy':
        text(lang,'privacyFlow').forEach((item,index)=>samples.append(el('p','',`${index+1}. ${item}`)));
        samples.append(button(text(lang,'demoAction'),'simulate'));
        break;
      case 'notifications':
        text(lang,'notifications').forEach((item,index)=>samples.append(el('p','',`DEMO-NOTICE-${index+1} · ${item}`)));
        break;
      case 'onboarding':
        addChecklist(samples, lang, feature, text(lang,'onboarding'));
        break;
      case 'preferences':
        samples.append(el('p','',text(lang,'prefHint')),button(text(lang,'demoAction'),'simulate'));
        break;
      default:
        samples.append(el('p','',text(lang,'synthetic')));
    }
    if (volatileState.message) samples.append(el('p','module-result',text(lang,volatileState.message)));
    card.append(samples);
    root.append(card, note(lang, text(lang, 'notConnected'), text(lang, 'noRealDecision')));
    return root;
  }

  function render(lang) {
    const feature = features.find(item => item.id === activeFeature);
    return feature ? renderDetail(lang, feature) : renderGallery(lang);
  }
  document.addEventListener('click', event => {
    const target = event.target.closest('[data-module-action]');
    if (!target) return;
    const action = target.dataset.moduleAction;
    if (action === 'open') activeFeature = target.dataset.feature;
    else if (action === 'back') activeFeature = null;
    else if (action === 'ack') volatileState.ack = true;
    else if (action === 'ticket') {
      volatileState.ticketCount = (volatileState.ticketCount || 0) + 1;
      volatileState.ticketCategory = document.querySelector('[data-ticket-select]')?.value || 'general';
    }
    else if (action === 'suggestion') volatileState.message = 'simulated';
    else if (action === 'question') volatileState.answer = Number(target.dataset.question);
    else if (action === 'interest') volatileState.message = 'applied';
    else if (action === 'simulate') volatileState.message = 'simulated';
    else if (action === 'export') {
      const csv = 'sample_category,count\nCATEGORY-A,12\nCATEGORY-B,7\nCATEGORY-C,3\n';
      const url = URL.createObjectURL(new Blob([csv], {type:'text/csv;charset=utf-8'}));
      const link = document.createElement('a'); link.href = url; link.download = 'inwp-synthetic-aggregate-sample.csv'; link.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      volatileState.message = 'exportDone';
    }
    document.dispatchEvent(new CustomEvent('inwp:module-render'));
  });

  window.INWP_MODULE_HUB = { features, render, messages };
})();
