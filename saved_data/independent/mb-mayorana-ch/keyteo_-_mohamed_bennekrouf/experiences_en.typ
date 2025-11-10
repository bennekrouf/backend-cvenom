#import "template.typ": conf, date, dated_experience, experience_details, section, get_text

#let get_key_insights() = {
  (
    "Over 15 years of experience in software development",
    "Expertise in Java, Node.js, Angular, React, and DevOps tools",
    "Strong craftsman and product-oriented experiences",
    "Solid DevOps background, particularly in continuous integration and related tools",
    "Languages & Frameworks: Java, JavaScript/TypeScript, C, Rust, Spring Boot, Node.js, AngularJS/Angular, React/React Native",
    "Databases: MongoDB, Oracle, DB2",
    "DevOps: Jenkins, Azure DevOps, OpenShift, Kubernetes, Docker, Bitbucket, GitHub Actions",
    "Cloud: GCP, AWS (VPS, S3)",
    "Systems & Middleware: MQ-Series, CICS, MVS-DB2, Kerberos",
    "Tools: Git, NVIM, VSCode",
    "Methodologies: Agile, Scrum",
    "French: Mother tongue",
    "English: Fluent",
  )
}

#let get_work_experience() = {
  [
    == KEYTEO SA, Switzerland
    #dated_experience(
      "Technical Lead",
      date: "November 2022 - Present (2 years and 9 months)",
      description: "Mission for the DGNSI – Application Integrator / DevOps: Modernization of the Etat de Vaud application portfolio through migration to homemade container solution or OpenShift.",
      content: [
        #experience_details(
          "Coached teams to migrate from git trunk-based to feature branch approach for improved deployment and quality."
        )
        #experience_details(
          "Implemented Docker, Jenkins, GitHub Actions, Bitbucket Pipelines; migrated a Maven/Nexus ecosystem."
        )
        #experience_details(
          "Developed a technical debt analysis tool for insights on upgrade requirements."
        )
        #experience_details(
          "Migrated C programs to RedHat 9 as part of infrastructure modernization."
        )
      ]
    )

    == Total Oil Trading SA
    #dated_experience(
      "Lead Fullstack Developer",
      date: "N/A",
      description: "Implemented continuous integration and provided support to the Fullstack team on Git and CI best practices.",
      content: [
        #experience_details(
          "Refactored Node.js/React codebase with hexagonal architecture."
        )
        #experience_details(
          "Integrated Kerberos authentication for native mobile applications."
        )
        #experience_details(
          "Set up CI with Docker Swarm and Azure DevOps for custom application deployment."
        )
        #experience_details(
          "Promoted CI/CD best practices through pull requests, build automation, and code reviews."
        )
      ]
    )

    == Startups Concreet.ch & Ejust.law
    #dated_experience(
      "Lead Developer",
      date: "January 2018 - November 2022 (4 years and 11 months)",
      description: "Context: Scheduling and eJustice applications for the construction industry and online arbitration.",
      content: [
        #experience_details(
          "Refactored eJust backend to support multi-tenancy and integrate a new mediation workflow."
        )
        #experience_details(
          "Defined technology stack and backend foundations for the Concreet application."
        )
        #experience_details(
          "Developed email authentication module using universal links in React Native."
        )
        #experience_details(
          "Refactored mobile app to integrate Redux."
        )
        #experience_details(
          "Developed Node.js plugins for Twilio, PostMark, and Zapier."
        )
        #experience_details(
          "Led the development team with daily meetings, code reviews, and technical decision-making."
        )
        #experience_details(
          "Set up DevOps infrastructure with Bitbucket CI/CD pipelines, Bash scripts, AWS S3 integration, and PM2 process management."
        )
      ]
    )

    == Service Company, Switzerland
    #dated_experience(
      "Senior Developer",
      date: "November 2015 - December 2017 (2 years)",
      description: "Maintained Java Spring Boot/AngularJS applications for Rolex and PMI.",
      content: [
        #experience_details(
          "Maintained a portfolio of applications based on Angular/AngularJS and Java."
        )
        #experience_details(
          "Led a Node.js initiative to promote best practices."
        )
      ]
    )

    == SASS company
    #dated_experience(
      "Lead Frontend Developer",
      date: "May 2014 - November 2015 (1 year and 6 months)",
      description: "Digital platform for biotech and pharma companies.",
      content: [
        #experience_details(
          "Led the frontend team in developing an enterprise application used by over 5,000 professionals."
        )
        #experience_details(
          "Developed architectural components and user interface components."
        )
        #experience_details(
          "Analyzed and resolved scalability issues."
        )
      ]
    )

    == Service Company, France
    #dated_experience(
      "Software Developer / Architect",
      date: "1998 - 2013 (15 years)",
      description: "Missions for clients: Carrefour, EDF, Total, Canal+, RATP.",
      content: [
        #experience_details(
          "Business analysis and business proposals."
        )
        #experience_details(
          "Architecture development and coaching: security, caches, automated testing, routing, source control, and API calls."
        )
        #experience_details(
          "EDF project: Outsourcing of nuclear applications with 30 systems and 50 engineers."
        )
        #experience_details(
          "Responsible for 6 architects supervising 50 engineers, coordinating infrastructure changes, code quality review, and cross-functional tool configuration."
        )
        #experience_details(
          "Implemented a generic code quality platform based on Sonar."
        )
        #experience_details(
          "Banques Populaires project: Equinoxe program for retail banking information system overhaul."
        )
        #experience_details(
          "Developed the mortgage application and backend data persistence framework."
        )
        #experience_details(
          "Architecture optimization: session analysis, thread management, and CPU contention analysis."
        )
        #experience_details(
          "Banque générale du Luxembourg project: CARAT program for information system overhaul."
        )
        #experience_details(
          "Responsible for configuration and version management during assembly and integration phases."
        )
        #experience_details(
          "Developed a framework for collateral deposit operations based on J2EE models."
        )
        #experience_details(
          "Designed market order management system components."
        )
        #experience_details(
          "Hewlett-Packard project: PS², integrating PeopleSoft to redesign purchasing and inventory processes."
        )
        #experience_details(
          "Developed the inventory module integration."
        )
        #experience_details(
          "Functional analysis of purchasing and receiving processes post-spin-off for Agilent Inc."
        )
      ]
    )

  ]
}
