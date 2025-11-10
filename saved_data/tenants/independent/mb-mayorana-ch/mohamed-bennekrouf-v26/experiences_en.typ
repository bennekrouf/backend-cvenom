#import "template.typ": conf, date, dated_experience, experience_details, section, get_text

#let get_key_insights() = {
  (
    "Java",
    "Node.js",
    "Angular",
    "React",
    "DevOps",
    "Jenkins",
    "Azure DevOps",
    "OpenShift",
    "Kubernetes",
    "Docker",
    "Bitbucket",
    "GitHub Actions",
    "GCP",
    "AWS",
    "Spring Boot",
    "Scrum",
    "Agile",
    "PMP",
    "Scrum Master",
    "TypeScript",
    "C/Rust",
    "MongoDB",
    "Oracle",
    "DB2",
    "MQ-Series",
    "CICS",
    "MVS-DB2",
    "Kerberos",
    "Git",
    "NVIM",
    "VSCode",
    "French",
    "English",
  )
}

#let get_work_experience() = {
  [
    == KEYTEO SA, Suisse
    #dated_experience(
      "Intégrateur – Lead Technique - DevOps",
      date: "Novembre 2022 - Aujourd'hui (2 ans et 9 mois)",
      description: "Modernisation du parc applicatif de l’état de Vaud (400 applications) par migration vers OpenShift.",
      content: [
        #experience_details(
          "Développement d’un outil d’analyse de la dette technique pour accompagner les refontes logicielles."
        )
        #experience_details(
          "Contribution DevOps : ajout de Docker, Jenkins, GitHub Actions, Bitbucket Pipelines ; migration d’un écosystème Maven/Nexus."
        )
      ]
    )

    == Total Oil Trading SA
    #dated_experience(
      "Lead Développeur Fullstack",
      date: "Janvier 2018 - Novembre 2022 (4 ans et 11 mois)",
      description: "Mise en place de l’intégration continue et support aux développeurs dans l’équipe Fullstack (Node/react – 15 développeurs) pour les bonnes pratiques GIT / CI.",
      content: [
        #experience_details(
          "Refactorisation d’un codebase Node.js/React en adoptant une architecture hexagonale."
        )
        #experience_details(
          "Intégration de l’intégration continue (CI) avec Docker Swarm et Azure DevOps pour le déploiement d'applications sur mesure."
        )
      ]
    )

    == Startups Concreet.ch & Ejust.law
    #dated_experience(
      "Lead Développeur",
      date: "Novembre 2015 - Novembre 2022 (2 ans)",
      description: "Application de planification pour l'industrie de la construction afin de gérer l'intégralité du processus, de la soumission des devis à la livraison finale du projet. Application eJustice pour effectuer des arbitrages et des médiations en ligne.",
      content: [
        #experience_details(
          "Refactorisation du backend eJust (Java/Spring Boot) pour supporter le multi-tenant et intégrer un nouveau workflow de médiation."
        )
        #experience_details(
          "Développement du module d’authentification par e-mail via lien universel dans une application React Native."
        )
        #experience_details(
          "Développement de plugins Node.js pour Twilio (SMS), PostMark (emails), et Zapier (intégration Slack)."
        )
        #experience_details(
          "Mise en place de l’infrastructure DevOps : pipelines Bitbucket CI/CD, scripts Bash, intégration AWS S3, gestion des processus avec PM2."
        )
      ]
    )

    == Société de Services, Suisse
    #dated_experience(
      "Senior Développeur",
      date: "Novembre 2015 - Décembre 2017 (2 ans)",
      description: "Maintenance d'applications Java Spring Boot/AngularJS pour Rolex et PMI. Développé des POCs utilisant Beacon, détection GPS pour application mobile (React native, Native script et Hybrid/Ionic2).",
      content: [
        #experience_details(
          "Maintenance d'un portefeuille d'applications basées sur Angular/AngularJS et Java (Rolex, PMI, Inexto Inc.)."
        )
      ]
    )

    == Éditeur de logiciels
    #dated_experience(
      "Lead Frontend Développeur",
      date: "Mai 2014 - Novembre 2015 (1 an et 6 mois)",
      description: "Plateforme digitale pour les entreprises biotech et pharma.",
      content: [
        #experience_details(
          "Dirige l'équipe frontend pour développer une application d'entreprise utilisée par plus de 5000 professionnels de l'industrie pharmaceutique."
        )
        #experience_details(
          "Analysé et résolu les problèmes de scalabilité."
        )
      ]
    )

    == Société de Services, France
    #dated_experience(
      "Développeur / Architecte Logiciel",
      date: "1998 - 2013 (15 ans)",
      description: "Missions pour plusieurs clients : Carrefour, EDF, Total, Canal+, RATP.",
      content: [
        #experience_details(
          "Analyse commerciale et propositions commerciales."
        )
        #experience_details(
          "Responsable de 6 architectes supervisant les 50 ingénieurs : coordination des changements d'infrastructure (virtualisation, approbation de l'achat de serveurs, reporting financier), mise en place de l'examen de la qualité du code (sonar), configuration des outils transversaux (gestion des bugs, dépôt de source, etc)."
        )
        #experience_details(
          "Développement de l'application du cadre technique (imprimante asynchrone, outil de génération de code, cache de données)."
        )
      ]
    )

  ]
}
