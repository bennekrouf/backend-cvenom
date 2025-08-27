#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = Expérience Professionnelle
    == Mayorana
    #dated_experience(
      "Freelance - Développeur Principal",
      date: "2022 - Présent",
      content: [
        #experience_details(
          "Programme Rust pour une migration de données, développement d'un programme d'analyse de dette technique, développement d'un Agent IA (basé sur BERT) soutenu par des micro-services rust (API matcher, messagerie, client mail)"
        )
        #experience_details(
          "En tant que développeur blockchain expérimenté : intégration de portefeuilles dans une application, interaction avec des smart contracts BNB, développement d'une application de suivi en temps réel du token DAI, développement de pallet substrate et corrections de bugs substrate (blockchain musicale Allfeat), création d'un token SPL Solana (RIBH sur Raydium)"
        )
        #experience_details(
          "Développement d'applications mobiles avec contributions open-source (Voir les dépôts github.com/bennekrouf/mayo*). Développement d'une application d'apprentissage avec un rust / rocket / sled (Cf similar-sled / ribh.io)."
        )
      ]
    )
    
    == Concreet
    #dated_experience(
      "CTO - Responsable Ingénierie Logicielle",
      date: "2016 - 2021",
      description: "Plateforme SAAS utilisée pour gérer des projets immobiliers et suivre les activités des collaborateurs via une application mobile",
      content: [
        #experience_details(
          "Définition de la stack technique et initiation du développement des composants architecturaux de l'application : scaffolding, couche fondamentale backend (authentification, connecteurs de base de données)"
        )
        #experience_details(
          "Direction et mentorat de l'équipe de développement : réunion quotidienne, revue de code, discussions techniques, refactoring et optimisation du code. Configuration Devops : Pipelines pour CI/CD, scripting bash, intégration AWS S3, planificateur de processus PM2"
        )
      ]
    )
    
    == CGI
    #dated_experience(
      "Développeur Principal - Architecte",
      date: "2007 -- 2015",
      content: [
        #experience_details(
          "Direction d'une équipe frontend pour développer une application d'entreprise utilisée par plus de 5000 professionnels pharmaceutiques (inpart.io), développement de POCs utilisant Beacon, détection GPS pour application mobile"
        )
        #experience_details(
          "Analyse de données et architecture de base de données pour construire un nouveau système de scoring crédit (Services Coface)"
        )
      ]
    )
    
    == Accenture
    #dated_experience(
      "Consultant",
      date: "1998 -- 2006",
      content: [
        #experience_details(
          "Développement de composants architecturaux (proxy, cache, authentification) et de composants UI (Banques Populaires)"
        )
        #experience_details(
          "Développement d'un framework de persistance de données backend utilisant : MQ-Series / WebSphere / programmes COBOL (Banque Générale du Luxembourg)"
        )
      ]
    )
  ]
}

