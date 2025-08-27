#import "template.typ": conf, date, dated_experience, experience_details, section

#let get_work_experience() = {
  [
    = Work Experience
    == Mayorana
    #dated_experience(
      "Freelance - Lead developer",
      date: "2022 - Present",
      content: [
        #experience_details(
          "Rust program for a data migration, developed a technical debt analysis program, developed an AI Agent (BERT based) backed by rust micro-services (API matcher, messaging, mail client)"
        )
        #experience_details(
          "As seasoned blockchain developer: integrated wallets to an app, interacted with BNB smart contracts, developed a DAI token realtime tracking app, substrate pallet development and substrate bug fixes (Allfeat music blockchain), created a Solana SPL token (RIBH on Raydium)"
        )
        #experience_details(
          "Mobile apps development with open-source contributions (See github.com/bennekrouf/mayo* repositories). Developed a learning app with a Rust Rocket backend using a sled binary tree (See similar-sled) behind ribh.io project."
        )
      ]
    )
    
    == Concreet
    #dated_experience(
      "CTO - Software engineering lead",
      date: "2016 - 2021",
      description: "SAAS platform used to manage real estate projects and track collaborators activities through a mobile app",
      content: [
        #experience_details(
          "Defined the technical stack and initiated the application architecture components development: scaffolding, backend foundation layer (authentication, database connectors)"
        )
        #experience_details(
          "Led and mentored the development team: daily meeting, code review, technical discussions, code refactoring and optimization. Devops setup: Pipelines for CI/CD, bash scripting, AWS S3 integration, PM2 process scheduler"
        )
      ]
    )
    
    == CGI
    #dated_experience(
      "Lead developer - Architect",
      date: "2007 -- 2015",
      content: [
        #experience_details(
          "Led a frontend team to develop an enterprise application used by more than 5000 pharmaceutical professionals (inpart.io), developed POCs using Beacon, GPS detection for mobile app"
        )
        #experience_details(
          "Data analysis and database architecture to build a new credit scoring system (Coface services)"
        )
      ]
    )
    
    == Accenture
    #dated_experience(
      "Consultant",
      date: "1998 -- 2006",
      content: [
        #experience_details(
          "Developed architectural components (proxy, cache, authentication) and some UI components (Banques Populaires)"
        )
        #experience_details(
          "Developed a back-end data persistence framework using: MQ-Series / WebSphere / COBOL programs (Banque Générale du Luxembourg)"
        )
      ]
    )
  ]
}
