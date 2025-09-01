#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    #dated_experience(  
      "Technical Lead Developer",  
      date: "From 2022 to Present (2 years and 9 months)",
      company: "Mayorana - Rust / an AI startup",
      content: [  
        #experience_details(  
          "Developed Rust Substrate smart contracts with frontend for allfeat.com, a blockchain-based music service platform built on Substrate. Created a react-native mobile app for one customer to use these services."
        )
        #experience_details(  
          "Built an NLP/LLM based program that converts sentences to API calls. Based on Rust / Ollama / Deepseek R1 / Iggy messaging and Micro-services. See api0.ai."  
        )
        #experience_details(  
          "Developed a log library based on Rust/gRPC. See crate grpc_logger."  
        )  
      ]  
    )
    
    #dated_experience(  
      "CTO - Founding software engineering - Lead development teams",  
      date: "From 2016 to 2021 (5 years)",
      company: "Concreet & eJust - Startup CTO roles",
      description: "Led technical development for two startups: Concreet (real estate SAAS platform) and eJust (justice arbitration platform)",  
      content: [  
        #experience_details(  
          "Built Concreet's SAAS platform with Node.js for real estate project management, including mobile app for field collaboration tracking."  
        )  
        #experience_details(  
          "Migrated eJust's monolithic Java backend to a multi-tenant architecture for justice arbitration and mediation."  
        )
        #experience_details(  
          "Implemented core backend features, CI/CD pipelines, AWS infrastructure management, and automated deployments."  
        )  
      ]  
    )
    
    #dated_experience(
      "Lead Frontend Developer",
      date: "From 2012 to 2016 (4 years)",
      company: "Inpart.io",
      description: "SASS platform for pharmaceutical professionals",
      content: [
        #experience_details(
          "Led frontend development team using Angular and React for an enterprise application serving 5000+ pharmaceutical professionals."
        )
        #experience_details(
          "Developed mobile applications using PhoneGap for cross-platform deployment and offline functionality."
        )
      ]
    )
    
    #dated_experience(
      "Lead Developer - Architect",
      date: "From 2007 to 2015 (8 years)",
      company: "CGI",
      content: [
        #experience_details(
          "Developed POCs using Beacon and GPS detection technologies for mobile applications."
        )
        #experience_details(
          "Data analysis and database architecture to build a new credit scoring system (Coface services)."
        )
      ]
    )
    
    #dated_experience(
      "Consultant",
      date: "From 1998 to 2006 (8 years)",
      company: "Accenture",
      content: [
        #experience_details(
          "Developed architectural components (proxy, cache, authentication) and UI components (Banques Populaires)"
        )
        #experience_details(
          "Developed a back-end data persistence framework using: MQ-Series / WebSphere / COBOL programs (Banque Générale du Luxembourg)"
        )
      ]
    )
  ]
}
