#import "template.typ": conf, date, dated_experience, experience_details, section, get_text

#let get_key_insights() = {
  (
    "Java, Node.js, Angular, React, DevOps, Agile, Scrum, Cloud, GCP, AWS, Kubernetes, Docker, Git, Jenkins, Azure DevOps, OpenShift, Bitbucket, GitHub Actions",
    "15+ years of software development experience, with expertise in Java, Node.js, Angular, and React",
    "Strong DevOps culture, including CI/CD, Kubernetes, Docker, and Git",
    "Familiarity with agile environments (Scrum, Kanban) and rapid delivery cycles",
    "Proficiency in cloud technologies (GCP, AWS)",
    "Experience with various middleware systems (MQ-Series, CICS, MVS-DB2, Kerberos)",
    "Proficiency in programming languages (Java, JavaScript/TypeScript, C/Rust)",
    "Familiarity with databases (MongoDB, Oracle, DB2)",
    "Knowledge of tools (Git, NVIM, VSCode)",
    "Certifications (PMP, Scrum Master)",
    "Fluency in French and English",
  )
}

#let get_work_experience() = {
  [
    == KEYTEO SA, Suisse
    #dated_experience(
      "Intégrateur – Lead Technique - DevOps",
      date: "Novembre 2022 - Aujourd’hui (2 ans et 9 mois)",
      description: "Modernization of the Vaud state's application park (400 applications) by migration to OpenShift.",
      content: [
        #experience_details(
          "Development of a technical debt analysis tool to support software refactors."
        )
        #experience_details(
          "Migration of C programs to RedHat 9 in a context of infrastructure modernization."
        )
        #experience_details(
          "DevOps contributions: addition of Docker, Jenkins, GitHub Actions, Bitbucket Pipelines; migration of a Maven/Nexus ecosystem."
        )
      ]
    )

    == Total Oil Trading SA
    #dated_experience(
      "Lead Développeur Fullstack",
      date: "Contexte",
      description: "Implementation of continuous integration and support for developers in the Fullstack team (Node/React – 15 developers) for Git/CI best practices.",
      content: [
        #experience_details(
          "Refactorization of a Node.js/React codebase adopting a hexagonal architecture."
        )
        #experience_details(
          "Integration of Kerberos authentication for native mobile applications."
        )
        #experience_details(
          "Implementation of continuous integration (CI) with Docker Swarm and Azure DevOps for the deployment of custom applications."
        )
        #experience_details(
          "Promotion of CI/CD best practices via pull requests, automation of builds, and code reviews."
        )
      ]
    )

    == Startups Concreet.ch & Ejust.law
    #dated_experience(
      "Lead Développeur",
      date: "De Janvier 2018 à Novembre 2022 (4 ans et 11 mois)",
      description: "Planning application for the construction industry to manage the entire process, from submitting quotes to the final delivery of the project. Application eJustice for online arbitration and mediation.",
      content: [
        #experience_details(
          "Refactorization of the eJust backend (Java/Spring Boot) to support multi-tenancy and integrate a new mediation workflow."
        )
        #experience_details(
          "Definition of the technological stack and backend foundations (authentication, DB connectors) for the Concreet application."
        )
        #experience_details(
          "Development of the email authentication module via a universal link in a React Native application."
        )
        #experience_details(
          "Refactorization of the mobile app to integrate Redux."
        )
        #experience_details(
          "Development of Node.js plugins for Twilio (SMS), PostMark (emails), and Zapier (Slack integration)."
        )
        #experience_details(
          "Management of the development team: daily meetings, code reviews, technical arbitrations."
        )
        #experience_details(
          "Implementation of DevOps infrastructure: Bitbucket CI/CD pipelines, Bash scripts, AWS S3 integration, process management with PM2."
        )
      ]
    )

    == Société de Services, Suisse
    #dated_experience(
      "Senior Développeur",
      date: "De novembre 2015 à décembre 2017 (2 ans)",
      description: "Maintenance of Java Spring Boot/AngularJS applications for Rolex and PMI. Development of POCs using Beacon, GPS detection for mobile applications (React native, Native script, and Hybrid/Ionic2).",
      content: [
        #experience_details(
          "Maintenance of a portfolio of applications based on Angular/AngularJS and Java (Rolex, PMI, Inexto Inc.)."
        )
        #experience_details(
          "Led a Node.js initiative to promote best practices."
        )
      ]
    )

    == Éditeur de logiciels
    #dated_experience(
      "Lead Frontend Développeur",
      date: "De mai 2014 à novembre 2015 (1 an et 6 mois)",
      description: "Digital platform for biotech and pharma companies.",
      content: [
        #experience_details(
          "Led the frontend team to develop an enterprise application used by over 5000 pharmaceutical industry professionals."
        )
        #experience_details(
          "Developed architectural components (proxy, cache, authentications) and some user interface components."
        )
        #experience_details(
          "Analyzed and resolved scalability issues."
        )
      ]
    )

    == Société de Services, France
    #dated_experience(
      "Développeur / Architecte Logiciel",
      date: "De 1998 à 2013 (15 ans)",
      description: "Missions for several clients: Carrefour, EDF, Total, Canal+, RATP.",
      content: [
        #experience_details(
          "Commercial analysis and proposals."
        )
        #experience_details(
          "Development of architecture and coaching: security, caches, automated tests, routing, source control configuration (Git), initial user interfaces, and API calls."
        )
        #experience_details(
          "EDF - Outsourcing of nuclear applications. Outsourcing of IT for nuclear applications (approximately 30 systems / over 50 engineers)."
        )
        #experience_details(
          "Responsible for 6 architects supervising 50 engineers: coordination of infrastructure changes (virtualization, approval of server purchases, financial reporting), implementation of code quality examination (Sonar), configuration of cross-functional tools (bug management, source repository, etc)."
        )
        #experience_details(
          "Implementation of a generic code quality platform (based on Sonar), scheduled analysis, and calculation of contractual metrics."
        )
        #experience_details(
          "Development of the technical framework application (asynchronous printer, code generation tool, data cache)."
        )
      ]
    )

  ]
}
