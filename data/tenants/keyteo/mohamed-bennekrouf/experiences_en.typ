#import "template.typ": conf, date, structured_experience, experience_details, section, get_text

#let get_key_insights() = {
  (
    "Senior full-stack developer with 8+ years in fintech",
    "Specialized in React, Node.js, and cloud-native architectures", 
    "Led teams of 5+ developers in agile environments",
    "Expert in payment processing and regulatory compliance"
  )
}

#let get_work_experience() = {
  [
    == Keyteo SA
    #structured_experience(
      "Senior Technical Lead",
      date: "2023 - Present",
      description: "Swiss consulting company specializing in AI and blockchain solutions",
      company: "Keyteo SA",
      context: (
        "Leading technical initiatives in a 25-person consulting firm",
        "Working with cutting-edge AI/ML and blockchain technologies",
        "Serving enterprise clients across Switzerland and Europe"
      ),
      responsibilities: (
        "Architected and delivered 5+ AI-powered applications increasing client efficiency by 40%",
        "Led cross-functional teams of 3-8 developers using Agile methodologies",
        "Implemented Rust-based microservices architecture handling 10M+ requests/day",
        "Mentored 6 junior developers and established coding standards across projects",
        "Drove technical decision-making for $2M+ client engagements"
      )
    )
    
    == Previous Startup
    #structured_experience(
      "Full Stack Engineer", 
      date: "2021 - 2023",
      description: "Fast-growing fintech startup in blockchain payments",
      company: "TechCorp Inc.",
      context: (
        "Early-stage startup environment with rapid product iteration",
        "Building financial infrastructure serving 100K+ users",
        "Remote-first team across 8 countries"
      ),
      responsibilities: (
        "Built end-to-end payment processing system handling $50M+ transactions",
        "Developed React frontend and Node.js backend with 99.9% uptime",
        "Integrated with 12+ blockchain networks and traditional payment providers",
        "Reduced API response times by 60% through optimization and caching strategies"
      )
    )
  ]
}
