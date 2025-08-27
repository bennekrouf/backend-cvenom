
#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills
#import "experiences_en.typ" : get_work_experience

#let details = toml("cv_params.toml")


// don't forget this
#show: doc => conf(details, doc)

#get_work_experience()

= Technical Skills
#show_skills(
  (
    "Languages": ("Rust (3y)", "Typescript (5y)", "JavaScript (>10y)", "Java (>10y)"),
    "Frameworks": ("NodeJS", "React", "Angular"),
    "Others": ("gRPC", "Git", "Linux", "OpenShift", "MongoDB", "SQL")
  )
)

= Certifications & Education
#dated_experience(
  "OpenShift - Administration I",
  date: "(2024)"
)
#dated_experience(
  "Computer Science University Degree (Programming) - Lyon Claude Bernard",
  date: "(1998)"
)
#dated_experience(
  "Science high-school diploma – Lycée La Pléiade",
  date: "(1996)"
)

= Languages
#experience_details(
  "French (Mother tongue)",
  "English (Fluent)",
)
