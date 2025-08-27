#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = الخبرة المهنية
    == مايورانا
    #dated_experience(
      "مطور رئيسي - عمل حر",
      date: "٢٠٢٢ - حتى الآن",
      content: [
        #experience_details(
          "برنامج Rust لترحيل البيانات، تطوير برنامج تحليل الديون التقنية، تطوير عميل ذكاء اصطناعي (مبني على BERT) مدعوم بخدمات مصغرة بلغة Rust (مطابقة API، المراسلة، عميل البريد)"
        )
        #experience_details(
          "كمطور blockchain متمرس: دمج المحافظ في التطبيق، التفاعل مع عقود BNB الذكية، تطوير تطبيق تتبع عملة DAI في الوقت الفعلي، تطوير وإصلاح أخطاء substrate (بلوكتشين Allfeat للموسيقى)، إنشاء عملة Solana SPL (RIBH على Raydium)"
        )
        #experience_details(
          "تطوير تطبيقات الهاتف المحمول مع مساهمات مفتوحة المصدر (انظر مستودعات github.com/bennekrouf/mayo*). تطوير تطبيق تعليمي مع خلفية Rust Rocket باستخدام شجرة ثنائية sled (انظر similar-sled) خلف مشروع ribh.io"
        )
      ]
    )
    
    == كونكريت
    #dated_experience(
      "المدير التقني - قائد هندسة البرمجيات",
      date: "٢٠١٦ - ٢٠٢١",
      description: "منصة SAAS تستخدم لإدارة المشاريع العقارية وتتبع أنشطة المتعاونين من خلال تطبيق الهاتف المحمول",
      content: [
        #experience_details(
          "تحديد المجموعة التقنية وبدء تطوير مكونات هيكل التطبيق: الهيكل الأساسي، طبقة الخلفية الأساسية (المصادقة، موصلات قواعد البيانات)"
        )
        #experience_details(
          "قيادة وتوجيه فريق التطوير: اجتماع يومي، مراجعة الكود، مناقشات تقنية، إعادة هيكلة وتحسين الكود. إعداد Devops: خطوط الأنابيب لـ CI/CD، برمجة bash، تكامل AWS S3، مجدول العمليات PM2"
        )
      ]
    )
    
    == CGI
    #dated_experience(
      "مطور رئيسي - مهندس معماري",
      date: "٢٠٠٧ -- ٢٠١٥",
      content: [
        #experience_details(
          "قيادة فريق واجهة أمامية لتطوير تطبيق مؤسسي يستخدمه أكثر من ٥٠٠٠ متخصص في المجال الصيدلاني (inpart.io)، تطوير نماذج أولية باستخدام Beacon، كشف GPS لتطبيق الهاتف المحمول"
        )
        #experience_details(
          "تحليل البيانات وهندسة قواعد البيانات لبناء نظام تصنيف ائتماني جديد (خدمات Coface)"
        )
      ]
    )
    
    == أكسنتشر
    #dated_experience(
      "مستشار",
      date: "١٩٩٨ -- ٢٠٠٦",
      content: [
        #experience_details(
          "تطوير مكونات معمارية (proxy، cache، المصادقة) وبعض مكونات واجهة المستخدم (Banques Populaires)"
        )
        #experience_details(
          "تطوير إطار عمل لحفظ البيانات في الخلفية باستخدام: MQ-Series / WebSphere / برامج COBOL (Banque Générale du Luxembourg)"
        )
      ]
    )
  ]
}

