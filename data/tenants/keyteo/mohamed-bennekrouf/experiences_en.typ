#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    == Mayorana - Rust / an AI startup
    #dated_experience(  
      "Technical Lead Developer",  
      date: "2022 - Present",  
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
    
    == Concreet & eJust - Startup CTO roles
    #dated_experience(  
      "CTO - Founding software engineering - Lead development teams",  
      date: "2016 - 2021",  
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
    
    == Inpart.io
    #dated_experience(
      "Lead Frontend Developer",
      date: "2012 - 2016",
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
    
    == CGI
    #dated_experience(
      "Lead Developer - Architect",
      date: "2007 - 2015",
      content: [
        #experience_details(
          "Developed POCs using Beacon and GPS detection technologies for mobile applications."
        )
        #experience_details(
          "Data analysis and database architecture to build a new credit scoring system (Coface services)."
        )
      ]
    )
    
    == Accenture
    #dated_experience(
      "Consultant",
      date: "1998 - 2006",
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
