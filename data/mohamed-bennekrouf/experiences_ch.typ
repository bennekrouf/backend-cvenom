#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = 工作经验
    == Mayorana
    #dated_experience(
      "自由职业 - 首席开发工程师",
      date: "2022 - 至今",
      content: [
        #experience_details(
          "使用Rust开发数据迁移程序，开发技术债务分析程序，开发基于BERT的人工智能代理，由Rust微服务支持（API匹配器、消息系统、邮件客户端）"
        )
        #experience_details(
          "作为资深区块链开发者：将钱包集成到应用程序中，与BNB智能合约交互，开发DAI代币实时跟踪应用，进行substrate模块开发和错误修复（Allfeat音乐区块链），创建Solana SPL代币（Raydium上的RIBH）"
        )
        #experience_details(
          "开发移动应用并贡献开源项目（参见github.com/bennekrouf/mayo*系列代码库）。使用Rust Rocket后端开发学习应用程序，在ribh.io项目中使用sled二叉树（参见similar-sled）"
        )
      ]
    )
    
    == Concreet
    #dated_experience(
      "技术总监 - 软件工程主管",
      date: "2016 - 2021",
      description: "用于管理房地产项目并通过移动应用程序跟踪协作者活动的SAAS平台",
      content: [
        #experience_details(
          "确定技术栈并启动应用程序架构组件开发：脚手架搭建、后端基础层（身份验证、数据库连接器）"
        )
        #experience_details(
          "领导并指导开发团队：每日会议、代码审查、技术讨论、代码重构和优化。DevOps配置：CI/CD流水线、bash脚本编写、AWS S3集成、PM2进程调度器"
        )
      ]
    )
    
    == CGI
    #dated_experience(
      "首席开发工程师 - 架构师",
      date: "2007 -- 2015",
      content: [
        #experience_details(
          "领导前端团队开发供5000多名制药专业人员使用的企业应用程序（inpart.io），使用Beacon开发概念验证项目，开发移动应用的GPS检测功能"
        )
        #experience_details(
          "进行数据分析和数据库架构设计，构建新的信用评分系统（Coface服务）"
        )
      ]
    )
    
    == Accenture
    #dated_experience(
      "咨询顾问",
      date: "1998 -- 2006",
      content: [
        #experience_details(
          "开发架构组件（代理、缓存、身份验证）和一些用户界面组件（大众银行）"
        )
        #experience_details(
          "使用MQ-Series / WebSphere / COBOL程序开发后端数据持久化框架（卢森堡通用银行）"
        )
      ]
    )
  ]
}

