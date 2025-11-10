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
    "Certifications: OpenShift Administration I, PMP, Scrum Master",
  )
}

#let get_work_experience() = {
  [
    == KEYTEO SA, Switzerland
    #dated_experience(
      "Technical Lead",
      date: "November 2022 - Present (2 years and 9 months)",
      description: "Mission for the DGNSI – Application Integrator / DevOps",
      content: [
        #experience_details(
          "Modernization of the Etat de Vaud application portfolio through migration to a homemade container solution or OpenShift."
        )
      ]
    )

    == Total Oil Trading SA
    #dated_experience(
      "Lead Fullstack Developer",
      date: "N/A",
      description: "Mission for Total Oil Trading SA",
      content: [
        #experience_details(
          "Implemented continuous integration and provided support to the Fullstack team (Node/React – 15 developers) on Git and CI best practices."
        )
      ]
    )

    == Startups Concreet.ch & Ejust.law
    #dated_experience(
      "Lead Developer",
      date: "January 2018 - November 2022 (4 years and 11 months)",
      description: "Context: Scheduling and eJustice applications for the construction and legal industries.",
      content: [
        #experience_details(
          "Refactored the eJust backend (Java/Spring Boot) to support multi-tenancy and integrate a new mediation workflow."
        )
        #experience_details(
          "Set up DevOps infrastructure: Bitbucket CI/CD pipelines, Bash scripts, AWS S3 integration, and process management with PM2."
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
          "Maintained a portfolio of applications based on Angular/AngularJS and Java (Rolex, PMI, Inexto Inc.)."
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
          "Led the frontend team in developing an enterprise application used by over 5,000 professionals in the pharmaceutical industry."
        )
      ]
    )

    == Service Company, France
    #dated_experience(
      "Software Developer / Architect",
      date: "1998 - 2013 (15 years)",
      description: "Missions for several clients: Carrefour, EDF, Total, Canal+, RATP.",
      content: [
        #experience_details(
          "Business analysis and business proposals."
        )
        #experience_details(
          "Project: EDF - Outsourcing of nuclear applications."
        )
        #experience_details(
          "Project: Banques Populaires – Equinoxe."
        )
        #experience_details(
          "Project: Banque générale du Luxembourg – CARAT."
        )
        #experience_details(
          "Project: Hewlett-Packard - PS²."
        )
      ]
    )

  ]
}
