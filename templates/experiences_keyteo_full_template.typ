#import "template.typ": conf, date, dated_experience, experience_details, section, get_text

// Enhanced experience function with structured context and responsibilities
#let structured_experience_full(title, date: none, description: none, company: none, context_info: none, responsibilities: none) = {
  [
    #block(
      stroke: (bottom: 0.5pt + rgb("#14A4E6")),
      inset: (bottom: 5pt),
      width: 100%,
      grid(
        columns: (1fr, auto),
        align: (left, right),
        [
          #text(size: 9pt, fill: rgb("#757575"), date) \
          #text(size: 11pt, weight: "bold", title)
        ],
        [
          #text(size: 11pt, weight: "bold", company)
        ]
      )
    )

    #if description != none [
      #text(weight: "regular", size: 10pt, description)
      #v(0.3em)
    ]

    #if context != none [
      #text(size: 10pt, weight: "bold", fill: rgb("#14A4E6"), "Contexte")
      #v(0.2em)
      
      // Handle context as array of bullet points or single text
      #if type(context) == array [
        #list(
          indent: 5pt,
          marker: text(fill: rgb("#14A4E6"), sym.bullet),
          ..context.map(item => text(size: 10pt, item))
        )
      ] else [
        #text(size: 10pt, context)
      ]
      #v(0.4em)
    ]

    #if responsibilities != none [
      #text(size: 10pt, weight: "bold", fill: rgb("#14A4E6"), "Responsabilités")
      #v(0.2em)
      
      // Handle responsibilities as dictionary with subsections
      #if type(responsibilities) == dictionary [
        #for (subsection, items) in responsibilities.pairs() [
          #text(size: 10pt, weight: "bold", [• #subsection])
          #v(0.1em)
          #if type(items) == array [
            #list(
              indent: 15pt,
              marker: text(fill: rgb("#14A4E6"), "◦"),
              ..items.map(item => text(size: 9pt, item))
            )
          ] else [
            #text(size: 9pt, indent: 15pt, items)
          ]
          #v(0.2em)
        ]
      ] else if type(responsibilities) == array [
        // Fallback to simple list if not structured
        #list(
          indent: 5pt,
          marker: text(fill: rgb("#14A4E6"), sym.bullet),
          ..responsibilities.map(item => text(size: 10pt, item))
        )
      ]
    ]

    #v(1em)
  ]
}

#let get_key_insights() = {
  (
    "Expert technique avec expérience confirmée en environnements startup et grandes entreprises",
    "Spécialiste des technologies modernes : Rust, Node.js, CI/CD, Docker, microservices", 
    "Leadership technique éprouvé avec capacité de coaching et d'industrialisation des pratiques",
    "Forte expertise en migration applicative et conception d'architectures évolutives"
  )
}

#let get_work_experience() = {
  [
    #structured_experience_full(
      "Développeur Backend Senior / Tech Lead",
      date: "Janvier 2023 - Décembre 2023",
      company: "Totsa SA",
      context_info: [Mission au sein de *Totsa SA (Trading & Supply, secteur pétrole & gaz)* visant à *moderniser les systèmes backend, renforcer les pratiques CI/CD et adapter une application de conformité multi-tenant*. Cette mission a nécessité une collaboration étroite avec les équipes de développement, d'administration système et DevOps afin d'assurer des déploiements sécurisés, évolutifs et une industrialisation des pratiques de développement.],
      responsibilities: (
        "Refonte backend": (
          "Conduite de la *refonte d'un backend Node.js* afin de supporter les appels d'applications mobiles avec une meilleure performance et fiabilité.",
        ),
        "CI/CD et pratiques de développement": (
          "Mise en place et industrialisation des *chaînes CI/CD*.",
          "Coaching des développeurs sur les bonnes pratiques *Git* : *pull requests, code reviews, stratégies de branches*.",
        ),
        "Déploiement & coordination infrastructure": (
          "Support direct et coordination avec les équipes *systèmes et DevOps* pour le déploiement des applications sous *Docker Swarm*.",
        ),
        "Migration applicative & conception technique": (
          "Conception technique et coordination de la *migration de l'application de conformité* (domaine pétrolier) afin de supporter un nouveau tenant : l'*entité gaz*.",
        ),
      )
    )
    
    #structured_experience_full(
      "Développeur Full Stack",
      date: "Mars 2022 - Décembre 2022",
      company: "Startup Innovante",
      context_info: (
        "Développement d'une plateforme SaaS multi-tenant avec architecture microservices",
        "Équipe de 5 développeurs en méthodologie agile",
        "Technologies: Rust, React, PostgreSQL, Docker"
      ),
      responsibilities: (
        "Architecture & développement": (
          "Conception et développement de l'*API backend en Rust* avec gestion multi-tenant",
          "Implémentation de l'*authentification JWT* et gestion des rôles",
          "Optimisation des performances avec *Redis* pour le cache"
        ),
        "DevOps & déploiement": (
          "Mise en place du *pipeline CI/CD avec GitHub Actions*",
          "Configuration du déploiement *Docker Swarm* en production",
          "Monitoring avec *Prometheus* et *Grafana*"
        ),
        "Encadrement technique": (
          "Mentoring de 2 développeurs junior",
          "Code reviews et définition des standards de développement"
        ),
      )
    )
  ]
}
