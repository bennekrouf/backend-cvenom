#import "template.typ": conf, date, structured_experience, experience_details, section, get_text

#let get_key_insights() = {
  (
    "Développeur senior full-stack avec 8+ ans d'expérience en fintech",
    "Spécialisé en React, Node.js et architectures cloud-natives",
    "Direction d'équipes de 5+ développeurs en environnements agiles", 
    "Expert en traitement des paiements et conformité réglementaire"
  )
}


#let get_work_experience() = {
  [
    // == Mayorana - Rust / une startup IA
    #structured_experience(  
      "Développeur Technical Lead",  
      date: "De 2022 à aujourd'hui (2 ans et 9 mois)",
      company: "Mayorana - Rust / une startup IA",
      context_info: (
        "Startup IA spécialisée dans les services musicaux blockchain et les solutions NLP/LLM",
        "Direction du développement technique utilisant Rust, Substrate et les technologies IA modernes"
      ),
      responsibility_list: (
        "Développement de smart contracts Rust Substrate avec frontend pour allfeat.com, une plateforme de services musicaux basée sur blockchain Substrate. Création d'une application mobile react-native pour un client utilisant ces services.",
        "Construction d'un programme basé sur NLP/LLM qui convertit les phrases en appels API. Basé sur Rust / Ollama / Deepseek R1 / messagerie Iggy et Micro-services. Voir api0.ai.",
        "Développement d'une bibliothèque de logs basée sur Rust/gRPC. Voir crate grpc_logger."
      )
    )
    
    // == Concreet & eJust - Rôles CTO Startup
    #structured_experience(  
      "CTO - Direction d'équipes de développement",  
      date: "De 2016 à 2021 (5 ans)",
      company: "Concreet & eJust - Rôles CTO Startup",
      context_info: (
        "Rôle de CTO gérant la stratégie technique pour deux startups SAAS B2B",
        "Responsabilité full-stack de la conception architecturale au déploiement et à la mise à l'échelle"
      ),
      responsibility_list: (
        "Construction de la plateforme SAAS de Concreet avec Node.js pour la gestion de projets immobiliers, incluant une application mobile pour le suivi collaboratif sur le terrain.",
        "Migration du backend Java monolithique d'eJust vers une architecture multi-tenant pour l'arbitrage et la médiation judiciaires.",
        "Implémentation des fonctionnalités backend principales, pipelines CI/CD, gestion d'infrastructure AWS et déploiements automatisés."
      )
    )
    
    // == Inpart.io
    #structured_experience(
      "Développeur Frontend Lead",
      date: "De 2012 à 2016 (4 ans)",
      company: "Inpart.io",
      context_info: (
        "Plateforme SAAS d'entreprise servant plus de 5000 professionnels pharmaceutiques",
        "Direction d'équipe de développement frontend dans un environnement startup en croissance rapide"
      ),
      responsibility_list: (
        "Direction de l'équipe de développement frontend utilisant Angular et React pour une application d'entreprise servant plus de 5000 professionnels pharmaceutiques.",
        "Développement d'applications mobiles utilisant PhoneGap pour le déploiement multi-plateforme et les fonctionnalités hors ligne."
      )
    )
    
    // == CGI
    #structured_experience(
      "Développeur Lead - Architecte",
      date: "De 2007 à 2015 (8 ans)",
      company: "CGI",
      context_info: (
        "Environnement de conseil à grande échelle travaillant sur des projets clients d'entreprise",
        "Spécialisation dans le secteur des services financiers avec systèmes de scoring crédit"
      ),
      responsibility_list: (
        "Développement de POCs utilisant les technologies de détection Beacon et GPS pour les applications mobiles.",
        "Analyse de données et architecture de base de données pour construire un nouveau système de scoring crédit (services Coface)."
      )
    )
    
    // == Accenture
    #structured_experience(
      "Consultant",
      date: "De 1998 à 2006 (8 ans)",
      company: "Accenture",
      context_info: (
        "Cabinet de conseil mondial travaillant avec de grandes institutions financières",
        "Projets d'intégration système et d'architecture de niveau entreprise"
      ),
      responsibility_list: (
        "Développement de composants architecturaux (proxy, cache, authentification) et composants UI (Banques Populaires)",
        "Développement d'un framework de persistance de données back-end utilisant : MQ-Series / WebSphere / programmes COBOL (Banque Générale du Luxembourg)"
      )
    )
  ]
}
